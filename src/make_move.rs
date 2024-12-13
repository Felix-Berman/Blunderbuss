use crate::{
    bitboard::Bitboard,
    move_types::{Move, MoveKind::*},
    piece::{Colour::*, Piece::*},
    position::{Castling, Position},
    square::Square,
    zobrist::ZOBRIST_CODES,
};

impl Position {
    pub fn make_move(&mut self, mv: Move) {
        self.halfmove_clk += 1;
        self.ply += 1;
        if let Some(sq) = self.ep_sq() {
            self.ep_bb = Bitboard::EMPTY;
            self.hash ^= ZOBRIST_CODES.en_passant(sq);
        }

        let us = self.active_colour;
        let them = !us;

        let from_bb = Bitboard::from(mv.from);
        let to_bb = Bitboard::from(mv.to);
        let from_to_bb = from_bb | to_bb;

        self.pieces[mv.piece] ^= from_to_bb;
        self.occupancy[us] ^= from_to_bb;
        self.hash ^= from_to_bb.fold(0, |acc, sq| acc ^ ZOBRIST_CODES.piece(mv.piece, sq));

        match mv.kind {
            Quiet => (),
            Capture(piece) => {
                self.pieces[piece] ^= to_bb;
                self.occupancy[them] ^= to_bb;
                self.halfmove_clk = 0;
                self.last_irreversible_ply = self.ply;
                self.hash ^= ZOBRIST_CODES.piece(piece, mv.to);
            }
            Promotion(piece) => {
                self.pieces[mv.piece] ^= to_bb;
                self.pieces[piece] ^= to_bb;
                self.hash ^=
                    ZOBRIST_CODES.piece(mv.piece, mv.to) ^ ZOBRIST_CODES.piece(piece, mv.to);
            }
            PromotionCapture(promo_piece, cap_piece) => {
                self.pieces[mv.piece] ^= to_bb;
                self.pieces[promo_piece] ^= to_bb;
                self.pieces[cap_piece] ^= to_bb;
                self.occupancy[them] ^= to_bb;
                self.hash ^= ZOBRIST_CODES.piece(cap_piece, mv.to);
                self.hash ^=
                    ZOBRIST_CODES.piece(mv.piece, mv.to) ^ ZOBRIST_CODES.piece(promo_piece, mv.to);
            }
            DoublePush => {
                self.ep_bb = (from_to_bb << 8) & (from_to_bb >> 8);
                self.hash ^= ZOBRIST_CODES.en_passant(self.ep_sq().unwrap());
            }
            EnPassant => {
                let captured_pawn = (from_bb << 1 | from_bb >> 1) & (to_bb << 8 | to_bb >> 8);
                self.pieces[Pawn(them)] ^= captured_pawn;
                self.occupancy[them] ^= captured_pawn;
                self.hash ^= ZOBRIST_CODES.piece(Pawn(them), captured_pawn.bitscan().unwrap());
            }
            Castle(castling) => {
                let from_to_bb = match castling {
                    Castling::W_KINGSIDE => Bitboard::from(Square::F1) | Bitboard::from(Square::H1),
                    Castling::W_QUEENSIDE => {
                        Bitboard::from(Square::D1) | Bitboard::from(Square::A1)
                    }
                    Castling::B_KINGSIDE => Bitboard::from(Square::F8) | Bitboard::from(Square::H8),
                    Castling::B_QUEENSIDE => {
                        Bitboard::from(Square::D8) | Bitboard::from(Square::A8)
                    }
                    _ => unreachable!("Invalid Castling code"),
                };
                self.pieces[Rook(us)] ^= from_to_bb;
                self.occupancy[us] ^= from_to_bb;
                self.hash ^= from_to_bb.fold(0, |acc, sq| acc ^ ZOBRIST_CODES.piece(Rook(us), sq));
                self.last_irreversible_ply = self.ply;
            }
        }

        self.hash ^= ZOBRIST_CODES.castling(self.castling);
        if let King(c) = mv.piece {
            let castling = match c {
                White => Castling::W_KINGSIDE + Castling::W_QUEENSIDE,
                Black => Castling::B_KINGSIDE + Castling::B_QUEENSIDE,
            };

            if self.castling.is_available(castling) {
                self.castling -= castling;
                self.last_irreversible_ply = self.ply;
            }
        }

        // remove castling for move from or to rook starting square
        for sq in from_to_bb & Bitboard::ROOKS {
            let castling = match sq {
                Square::H1 => Castling::W_KINGSIDE,
                Square::A1 => Castling::W_QUEENSIDE,
                Square::H8 => Castling::B_KINGSIDE,
                Square::A8 => Castling::B_QUEENSIDE,
                _ => unreachable!(),
            };

            if self.castling.is_available(castling) {
                self.castling -= castling;
                self.last_irreversible_ply = self.ply;
            }
        }
        self.hash ^= ZOBRIST_CODES.castling(self.castling);

        if let Pawn(_) = mv.piece {
            self.halfmove_clk = 0;
            self.last_irreversible_ply = self.ply;
        }

        self.active_colour = them;
        self.hash ^= ZOBRIST_CODES.colour();

        debug_assert_eq!(self.occupied(), {
            self.gen_occupancy();
            self.occupied()
        });
        debug_assert_eq!(self.hash, self.gen_zobrist_hash());
    }

    pub fn find_algebraic_move(&self, mv_str: &str) -> Option<Move> {
        let mut moves = self.gen_moves();

        moves.find(|&mv| mv.to_string() == mv_str.to_ascii_lowercase())
    }
}
