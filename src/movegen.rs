use crate::{
    bitboard::Bitboard,
    magic::{BISHOP_BITS, MAGICS, ROOK_BITS},
    move_types::{Move, MoveKind, MoveList},
    piece::{
        Colour::{self, *},
        Piece::*,
    },
    position::{Castling, Position},
    square::Square,
};

pub fn king_attacks(sq: Square) -> Bitboard {
    let king = Bitboard::from(sq);
    let mut attacks = Bitboard::EMPTY;
    attacks |= king << 7 & !Bitboard::H_FILE;
    attacks |= king << 8;
    attacks |= king << 9 & !Bitboard::A_FILE;
    attacks |= king << 1 & !Bitboard::A_FILE;
    attacks |= king >> 7 & !Bitboard::A_FILE;
    attacks |= king >> 8;
    attacks |= king >> 9 & !Bitboard::H_FILE;
    attacks |= king >> 1 & !Bitboard::H_FILE;
    attacks
}

pub fn knight_attacks(sq: Square) -> Bitboard {
    let knight = Bitboard::from(sq);
    let mut attacks = Bitboard::EMPTY;
    attacks |= knight << 6 & !Bitboard::H_FILE & !Bitboard::G_FILE;
    attacks |= knight << 15 & !Bitboard::H_FILE;
    attacks |= knight << 17 & !Bitboard::A_FILE;
    attacks |= knight << 10 & !Bitboard::A_FILE & !Bitboard::B_FILE;
    attacks |= knight >> 6 & !Bitboard::A_FILE & !Bitboard::B_FILE;
    attacks |= knight >> 15 & !Bitboard::A_FILE;
    attacks |= knight >> 17 & !Bitboard::H_FILE;
    attacks |= knight >> 10 & !Bitboard::H_FILE & !Bitboard::G_FILE;

    attacks
}

pub fn pawn_pushes(bb: Bitboard, side: Colour, occ: Bitboard) -> (Bitboard, Bitboard) {
    match side {
        White => {
            let single_push = (bb >> 8) & !occ;
            let double_push = (single_push >> 8) & !occ & Bitboard::RANK_4;
            (single_push, double_push)
        }
        Black => {
            let single_push = (bb << 8) & !occ;
            let double_push = (single_push << 8) & !occ & Bitboard::RANK_5;
            (single_push, double_push)
        }
    }
}

pub fn pawn_attacks(bb: Bitboard, side: Colour) -> Bitboard {
    match side {
        White => (bb >> 7 & !Bitboard::A_FILE) | (bb >> 9 & !Bitboard::H_FILE),
        Black => (bb << 7 & !Bitboard::H_FILE) | (bb << 9 & !Bitboard::A_FILE),
    }
}

pub fn pawn_captures(bb: Bitboard, side: Colour, opponent: Bitboard) -> [(Bitboard, Bitboard); 2] {
    match side {
        White => {
            let mut from_left = bb & !Bitboard::A_FILE;
            let mut from_right = bb & !Bitboard::H_FILE;
            let to_left = from_left >> 9 & opponent;
            let to_right = from_right >> 7 & opponent;
            from_left &= to_left << 9;
            from_right &= to_right << 7;
            [(from_left, to_left), (from_right, to_right)]
        }
        Black => {
            let mut from_left = bb & !Bitboard::A_FILE;
            let mut from_right = bb & !Bitboard::H_FILE;
            let to_left = from_left << 7 & opponent;
            let to_right = from_right << 9 & opponent;
            from_left &= to_left >> 7;
            from_right &= to_right >> 9;
            [(from_left, to_left), (from_right, to_right)]
        }
    }
}

fn rook_attacks(sq: Square, mut occ: Bitboard) -> Bitboard {
    occ &= MAGICS.rook_magics[sq as usize].mask;
    let mut occ = occ * MAGICS.rook_magics[sq as usize].magic;
    occ >>= 64 - ROOK_BITS[sq];

    MAGICS.rook_attacks[sq as usize][occ as usize]
}

fn bishop_attacks(sq: Square, mut occ: Bitboard) -> Bitboard {
    occ &= MAGICS.bishop_magics[sq as usize].mask;
    let mut occ = occ * MAGICS.bishop_magics[sq as usize].magic;
    occ >>= 64 - BISHOP_BITS[sq];

    MAGICS.bishop_attacks[sq as usize][occ as usize]
}

impl Position {
    pub fn gen_moves(&self) -> MoveList {
        let mut moves = MoveList::new();
        self.gen_captures(&mut moves);
        self.gen_quiet_moves(&mut moves);
        moves
    }

    pub fn is_sq_attacked_by(&self, sq: Square, side: Colour) -> bool {
        pawn_attacks(Bitboard::from(sq), !side).intersects(&self.pieces[Pawn(side)])
            || knight_attacks(sq).intersects(&self.pieces[Knight(side)])
            || king_attacks(sq).intersects(&self.pieces[King(side)])
            || bishop_attacks(sq, self.occupied())
                .intersects(&(self.pieces[Bishop(side)] | self.pieces[Queen(side)]))
            || rook_attacks(sq, self.occupied())
                .intersects(&(self.pieces[Rook(side)] | self.pieces[Queen(side)]))
    }

    pub fn is_check(&self, side: Colour) -> bool {
        let king = self.pieces[King(side)]
            .bitscan()
            .expect(&format!("Missing king {side}"));
        self.is_sq_attacked_by(king, !side)
    }

    pub fn gen_pawn_pushes(&self, moves: &mut MoveList) {
        let c = self.active_colour;
        let occ = self.occupied();
        let bb = self.pieces[Pawn(c)];

        let (to_single, to_double) = pawn_pushes(bb, c, occ);
        for to in to_single {
            let from = match c {
                White => to + 8u8,
                Black => to - 8u8,
            }
            .expect("pawn on back rank");

            debug_assert_eq!(
                self.piece_on(from),
                Some(Pawn(c)),
                "from: {}, to: {}",
                from,
                to
            );

            if to.rank() == Square::RANK_1 || to.rank() == Square::RANK_8 {
                for p in [Queen(c), Rook(c), Bishop(c), Knight(c)] {
                    moves.push(Move {
                        from,
                        to,
                        piece: Pawn(c),
                        kind: MoveKind::Promotion(p),
                    });
                }
            } else {
                moves.push(Move {
                    from,
                    to,
                    piece: Pawn(c),
                    kind: MoveKind::Quiet,
                });
            }
        }
        for to in to_double {
            let from = match c {
                White => to + 16u8,
                Black => to - 16u8,
            }
            .expect("pawn on back rank");

            debug_assert_eq!(
                self.piece_on(from),
                Some(Pawn(c)),
                "from: {}, to: {}",
                from,
                to
            );

            moves.push(Move {
                from,
                to,
                piece: Pawn(c),
                kind: MoveKind::DoublePush,
            });
        }
    }

    pub fn gen_pawn_captures(&self, moves: &mut MoveList) {
        let c = self.active_colour;
        let opponent = self.occupancy[!c];
        let bb = self.pieces[Pawn(c)];

        for (from_bb, to_bb) in pawn_captures(bb, c, opponent) {
            for (from, to) in from_bb.zip(to_bb) {
                let captured = self
                    .piece_on(to)
                    .expect("occupancy and piece bitboards out of sync");

                if to.rank() == Square::RANK_1 || to.rank() == Square::RANK_8 {
                    for p in [Queen(c), Rook(c), Bishop(c), Knight(c)] {
                        moves.push(Move {
                            from,
                            to,
                            piece: Pawn(c),
                            kind: MoveKind::PromotionCapture(p, captured),
                        });
                    }
                } else {
                    moves.push(Move {
                        from,
                        to,
                        piece: Pawn(c),
                        kind: MoveKind::Capture(captured),
                    });
                }
            }
        }
    }

    pub fn gen_quiet_moves(&self, moves: &mut MoveList) {
        let occ = self.occupied();
        self.gen_castling(moves);
        self.gen_pawn_pushes(moves);

        for (piece, bb) in self.iter_pieces_by_colour(self.active_colour) {
            for from in *bb {
                let to_bb = match piece {
                    Pawn(_) => continue,
                    Knight(_) => knight_attacks(from),
                    Bishop(_) => bishop_attacks(from, occ),
                    Rook(_) => rook_attacks(from, occ),
                    Queen(_) => bishop_attacks(from, occ) | rook_attacks(from, occ),
                    King(_) => king_attacks(from),
                } & !occ;

                for to in to_bb {
                    moves.push(Move {
                        from,
                        to,
                        piece,
                        kind: MoveKind::Quiet,
                    });
                }
            }
        }
    }

    pub fn gen_captures(&self, moves: &mut MoveList) {
        let occ = self.occupied();
        let opponent = self.occupancy[!self.active_colour];
        self.gen_en_passant(moves);
        self.gen_pawn_captures(moves);

        for (piece, bb) in self.iter_pieces_by_colour(self.active_colour) {
            for from in *bb {
                let to_bb = match piece {
                    Pawn(_) => continue,
                    Knight(_) => knight_attacks(from),
                    Bishop(_) => bishop_attacks(from, occ),
                    Rook(_) => rook_attacks(from, occ),
                    Queen(_) => bishop_attacks(from, occ) | rook_attacks(from, occ),
                    King(_) => king_attacks(from),
                } & opponent;

                for to in to_bb {
                    let captured = self
                        .piece_on(to)
                        .expect("occupancy and piece bitboards out of sync");

                    moves.push(Move {
                        from,
                        to,
                        piece,
                        kind: MoveKind::Capture(captured),
                    });
                }
            }
        }
    }

    fn gen_en_passant(&self, moves: &mut MoveList) {
        if let Some(to) = self.ep_sq() {
            let from_bb = pawn_attacks(Bitboard::from(to), !self.active_colour)
                & self.pieces[Pawn(self.active_colour)];
            for from in from_bb {
                moves.push(Move {
                    from,
                    to,
                    piece: Pawn(self.active_colour),
                    kind: MoveKind::EnPassant,
                });
            }
        }
    }

    fn gen_castling(&self, moves: &mut MoveList) {
        if self.castling.is_empty() {
            return;
        }

        let occ = self.occupied();

        match self.active_colour {
            White => {
                if self.castling.is_available(Castling::W_KINGSIDE)
                    && !occ.intersects(&(Bitboard::KINGSIDE_CASTLING << 56))
                    && !self.is_sq_attacked_by(Square::F1, Black)
                    && !self.is_sq_attacked_by(Square::G1, Black)
                    && !self.is_sq_attacked_by(Square::E1, Black)
                {
                    moves.push(Move {
                        from: Square::E1,
                        to: Square::G1,
                        piece: King(White),
                        kind: MoveKind::Castle(Castling::W_KINGSIDE),
                    });
                }
                if self.castling.is_available(Castling::W_QUEENSIDE)
                    && !occ.intersects(&(Bitboard::QUEENSIDE_CASTLING << 56))
                    && !self.is_sq_attacked_by(Square::D1, Black)
                    && !self.is_sq_attacked_by(Square::C1, Black)
                    && !self.is_sq_attacked_by(Square::E1, Black)
                {
                    moves.push(Move {
                        from: Square::E1,
                        to: Square::C1,
                        piece: King(White),
                        kind: MoveKind::Castle(Castling::W_QUEENSIDE),
                    });
                }
            }
            Black => {
                if self.castling.is_available(Castling::B_KINGSIDE)
                    && !occ.intersects(&(Bitboard::KINGSIDE_CASTLING))
                    && !self.is_sq_attacked_by(Square::F8, White)
                    && !self.is_sq_attacked_by(Square::G8, White)
                    && !self.is_sq_attacked_by(Square::E8, White)
                {
                    moves.push(Move {
                        from: Square::E8,
                        to: Square::G8,
                        piece: King(Black),
                        kind: MoveKind::Castle(Castling::B_KINGSIDE),
                    });
                }
                if self.castling.is_available(Castling::B_QUEENSIDE)
                    && !occ.intersects(&(Bitboard::QUEENSIDE_CASTLING))
                    && !self.is_sq_attacked_by(Square::D8, White)
                    && !self.is_sq_attacked_by(Square::C8, White)
                    && !self.is_sq_attacked_by(Square::E8, White)
                {
                    moves.push(Move {
                        from: Square::E8,
                        to: Square::C8,
                        piece: King(Black),
                        kind: MoveKind::Castle(Castling::B_QUEENSIDE),
                    });
                }
            }
        }
    }
}
