use std::fmt::Display;

use crate::{
    bitboard::{Bitboard, Square},
    magic::{BISHOP_BITS, MAGICS, ROOK_BITS},
    position::{
        CastlingFlags,
        Colour::{self, *},
        Piece, Position,
    },
    search::{mvv_lva, SearchInfo},
};
use itertools::Itertools;
use Piece::*;
use Square::*;

pub const MAX_MOVES: usize = 256;
const KING_MOVES: [u64; 64] = build_king_tbl();
const KNIGHT_MOVES: [u64; 64] = build_knight_tbl();

const fn build_king_tbl() -> [u64; 64] {
    let mut moves = [0; 64];
    let mut sq = 0;
    while sq < 64 {
        let king = 1 << sq;
        moves[sq] = king << 7 & !Bitboard::H_FILE.0
            | king << 8
            | king << 9 & !Bitboard::A_FILE.0
            | king << 1 & !Bitboard::A_FILE.0
            | king >> 7 & !Bitboard::A_FILE.0
            | king >> 8
            | king >> 9 & !Bitboard::H_FILE.0
            | king >> 1 & !Bitboard::H_FILE.0;
        sq += 1;
    }

    moves
}

const fn build_knight_tbl() -> [u64; 64] {
    let mut moves = [0; 64];
    let mut sq = 0;
    while sq < 64 {
        let knight = 1 << sq;
        moves[sq] = knight << 6 & !Bitboard::H_FILE.0 & !Bitboard::G_FILE.0
            | knight << 15 & !Bitboard::H_FILE.0
            | knight << 17 & !Bitboard::A_FILE.0
            | knight << 10 & !Bitboard::A_FILE.0 & !Bitboard::B_FILE.0
            | knight >> 6 & !Bitboard::A_FILE.0 & !Bitboard::B_FILE.0
            | knight >> 15 & !Bitboard::A_FILE.0
            | knight >> 17 & !Bitboard::H_FILE.0
            | knight >> 10 & !Bitboard::H_FILE.0 & !Bitboard::G_FILE.0;

        sq += 1;
    }
    moves
}

pub fn king_attacks(sq: Square) -> Bitboard {
    let king = Bitboard::from(sq);
    let mut attacks = Bitboard(0);
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
    let mut attacks = Bitboard(0);
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

pub fn pawn_attacks(sq: Square, side: Colour) -> Bitboard {
    let mut attacks = Bitboard(0);
    let pawn = Bitboard::from(sq);

    match side {
        Colour::White => {
            attacks |= pawn >> 7 & !Bitboard::A_FILE;
            attacks |= pawn >> 9 & !Bitboard::H_FILE;
        }
        Colour::Black => {
            attacks |= pawn << 7 & !Bitboard::H_FILE;
            attacks |= pawn << 9 & !Bitboard::A_FILE;
        }
    }

    attacks
}

pub fn pawn_pushes(sq: Square, side: Colour) -> Bitboard {
    let mut pushes = Bitboard(0);

    let direction = match side {
        White => -8,
        Black => 8,
    };

    if let Some(sq) = sq.add(direction) {
        pushes.set(sq);
    }

    pushes
}

fn rook_attacks(sq: Square, mut occ: Bitboard) -> Bitboard {
    occ &= MAGICS.rook_magics[sq as usize].mask;
    occ.0 *= MAGICS.rook_magics[sq as usize].magic;
    occ >>= 64 - ROOK_BITS[sq];

    MAGICS.rook_attacks[sq as usize][occ.0 as usize]
}

fn bishop_attacks(sq: Square, mut occ: Bitboard) -> Bitboard {
    occ &= MAGICS.bishop_magics[sq as usize].mask;
    occ.0 *= MAGICS.bishop_magics[sq as usize].magic;
    occ >>= 64 - BISHOP_BITS[sq];

    MAGICS.bishop_attacks[sq as usize][occ.0 as usize]
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub piece: Piece,
    pub kind: MoveKind,
}

impl Move {
    pub const NULL: Move = Move::new();

    pub const fn new() -> Move {
        Move {
            from: A1,
            to: A1,
            piece: Pawn(White),
            kind: MoveKind::Quiet,
        }
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let promotion = if let MoveKind::Promotion(p) | MoveKind::PromotionCapture(p, _) = self.kind
        {
            p.to_string()
        } else {
            "".to_string()
        };

        write!(f, "{}{}{}", self.from, self.to, promotion)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MoveKind {
    Quiet,
    Capture(Piece),
    DoublePawnPush,
    EnPassant,
    Castling(CastlingFlags),
    Promotion(Piece),
    PromotionCapture(Piece, Piece),
}

pub struct MoveList {
    pub moves: [Move; MAX_MOVES],
    pub sort_scores: [u8; MAX_MOVES],
    pub length: usize,
    pub curr: usize,
}

impl MoveList {
    pub fn new() -> MoveList {
        MoveList {
            moves: [Move::NULL; MAX_MOVES],
            sort_scores: [0; MAX_MOVES],
            length: 0,
            curr: 0,
        }
    }

    pub fn push(&mut self, mv: Move) {
        self.moves[self.length] = mv;
        self.length += 1;
    }

    pub fn pop(&mut self) -> Move {
        self.length -= 1;
        self.moves[self.length]
    }

    pub fn score(&mut self, ply: usize, info: &SearchInfo) {
        for i in 0..self.length {
            let mv = &mut self.moves[i];

            if info.current_branch[..ply] == info.triangular_pv[..ply]
                && info.triangular_pv[ply].is_some_and(|pv_mv| *mv == pv_mv)
            {
                self.sort_scores[i] += 100;
            }

            self.sort_scores[i] += mvv_lva(mv);
        }
    }
}

impl Iterator for MoveList {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr >= self.length {
            return None;
        }

        let mut next_best = self.moves[self.curr];
        let mut best_score = self.sort_scores[self.curr];

        for i in (self.curr + 1)..self.length {
            if self.sort_scores[i] > best_score {
                next_best = self.moves[i];
                best_score = self.sort_scores[i];
                self.moves[i] = self.moves[self.curr];
                self.sort_scores[i] = self.sort_scores[self.curr];
                self.moves[self.curr] = next_best;
                self.sort_scores[self.curr] = best_score;
            }
        }

        self.curr += 1;
        Some(next_best)
    }
}

impl Position {
    pub fn gen_moves(&self) -> MoveList {
        let mut moves = MoveList::new();
        self.gen_captures(&mut moves);
        self.gen_quiet_moves(&mut moves);
        moves
    }

    pub fn is_sq_attacked_by(&self, sq: Square, side: Colour) -> bool {
        pawn_attacks(sq, !side).intersects(self.pieces[Pawn(side)])
            || knight_attacks(sq).intersects(self.pieces[Knight(side)])
            || king_attacks(sq).intersects(self.pieces[King(side)])
            || bishop_attacks(sq, self.occupied())
                .intersects(self.pieces[Bishop(side)] | self.pieces[Queen(side)])
            || rook_attacks(sq, self.occupied())
                .intersects(self.pieces[Rook(side)] | self.pieces[Queen(side)])
    }

    pub fn is_check(&self, side: Colour) -> bool {
        let king = self.pieces[King(side)].get_lsb().expect("missing king");
        self.is_sq_attacked_by(king, !side)
    }

    pub fn gen_quiet_moves(&self, moves: &mut MoveList) {
        let occ = self.occupied();
        self.gen_castling(moves);

        for piece in Piece::iter_colour(self.turn) {
            for from in self.pieces[piece] {
                let bb = match piece {
                    Pawn(c) => {
                        self.gen_double_pawn_pushes(moves, c, from);
                        pawn_pushes(from, c)
                    }
                    Knight(_) => knight_attacks(from),
                    Bishop(_) => bishop_attacks(from, occ),
                    Rook(_) => rook_attacks(from, occ),
                    Queen(_) => bishop_attacks(from, occ) | rook_attacks(from, occ),
                    King(_) => king_attacks(from),
                } & !occ;

                if let Pawn(c) = piece {
                    let backranks = Bitboard(0xff | (0xff << 56));
                    if bb.intersects(backranks) {
                        let promotables = [Queen(c), Rook(c), Bishop(c), Knight(c)];
                        for p in promotables {
                            moves.push(Move {
                                from,
                                to: bb.get_lsb().unwrap(),
                                piece,
                                kind: MoveKind::Promotion(p),
                            });
                        }

                        continue;
                    }
                }

                for to in bb {
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
        let opponent = self.occupancy[!self.turn];
        self.gen_en_passant(moves);

        for piece in Piece::iter_colour(self.turn) {
            for from in self.pieces[piece] {
                let bb = match piece {
                    Pawn(c) => pawn_attacks(from, c),
                    Knight(_) => knight_attacks(from),
                    Bishop(_) => bishop_attacks(from, occ),
                    Rook(_) => rook_attacks(from, occ),
                    Queen(_) => bishop_attacks(from, occ) | rook_attacks(from, occ),
                    King(_) => king_attacks(from),
                } & opponent;

                for to in bb {
                    let captured = self.piece_on(to).unwrap();

                    if let Pawn(c) = piece {
                        if (A1..=H1).contains(&to) || (A8..=H8).contains(&to) {
                            let promotables = [Queen(c), Rook(c), Bishop(c), Knight(c)];
                            for p in promotables {
                                moves.push(Move {
                                    from,
                                    to,
                                    piece,
                                    kind: MoveKind::PromotionCapture(p, captured),
                                });
                            }
                            continue;
                        }
                    }

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

    pub fn gen_double_pawn_pushes(&self, moves: &mut MoveList, c: Colour, from: Square) {
        match (c, from as i8) {
            (White, 48..=55) => {
                let mut path = Bitboard::from(from);
                path >>= 8;
                path |= path >> 8;
                if self.occupied().intersects(path) {
                    return;
                }

                let to = from.add(-16).unwrap();
                moves.push(Move {
                    from,
                    to,
                    piece: Pawn(c),
                    kind: MoveKind::DoublePawnPush,
                })
            }
            (Black, 8..=15) => {
                let mut path = Bitboard::from(from);
                path <<= 8;
                path |= path << 8;
                if self.occupied().intersects(path) {
                    return;
                }

                let to = from.add(16).unwrap();
                moves.push(Move {
                    from,
                    to,
                    piece: Pawn(c),
                    kind: MoveKind::DoublePawnPush,
                })
            }
            _ => (),
        }
    }

    pub fn gen_en_passant(&self, moves: &mut MoveList) {
        if let Some(to) = self.en_passant {
            let from_bb = pawn_attacks(to, !self.turn) & self.pieces[Pawn(self.turn)];
            for from in from_bb {
                moves.push(Move {
                    from,
                    to,
                    piece: Pawn(self.turn),
                    kind: MoveKind::EnPassant,
                });
            }
        }
    }

    pub fn gen_castling(&self, moves: &mut MoveList) {
        if self.castling.is_empty() {
            return;
        }

        let occ = self.occupied();

        match self.turn {
            White => {
                if self.castling.contains(CastlingFlags::WK)
                    && occ.0 & 0x60 << 56 == 0
                    && !self.is_sq_attacked_by(F1, Black)
                    && !self.is_sq_attacked_by(G1, Black)
                    && !self.is_sq_attacked_by(E1, Black)
                {
                    moves.push(Move {
                        from: Square::E1,
                        to: Square::G1,
                        piece: King(White),
                        kind: MoveKind::Castling(CastlingFlags::WK),
                    });
                }
                if self.castling.contains(CastlingFlags::WQ)
                    && occ.0 & 0xe << 56 == 0
                    && !self.is_sq_attacked_by(D1, Black)
                    && !self.is_sq_attacked_by(C1, Black)
                    && !self.is_sq_attacked_by(E1, Black)
                {
                    moves.push(Move {
                        from: Square::E1,
                        to: Square::C1,
                        piece: King(White),
                        kind: MoveKind::Castling(CastlingFlags::WQ),
                    });
                }
            }
            Black => {
                if self.castling.contains(CastlingFlags::BK)
                    && occ.0 & 0x60 == 0
                    && !self.is_sq_attacked_by(F8, White)
                    && !self.is_sq_attacked_by(G8, White)
                    && !self.is_sq_attacked_by(E8, White)
                {
                    moves.push(Move {
                        from: Square::E8,
                        to: Square::G8,
                        piece: King(Black),
                        kind: MoveKind::Castling(CastlingFlags::BK),
                    });
                }

                if self.castling.contains(CastlingFlags::BQ)
                    && occ.0 & 0xe == 0
                    && !self.is_sq_attacked_by(D8, White)
                    && !self.is_sq_attacked_by(C8, White)
                    && !self.is_sq_attacked_by(E8, White)
                {
                    moves.push(Move {
                        from: Square::E8,
                        to: Square::C8,
                        piece: King(Black),
                        kind: MoveKind::Castling(CastlingFlags::BQ),
                    });
                }
            }
        }
    }
}
