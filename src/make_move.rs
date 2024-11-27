use crate::{
    bitboard::Bitboard,
    move_types::{Move, MoveKind::*},
    piece::{Colour::*, Piece::*},
    position::{Castling, Position},
    square::Square,
};

impl Position {
    pub fn make_move(&mut self, mv: Move) {
        self.halfmove_clk += 1;
        self.ep_bb = Bitboard::EMPTY;

        let us = self.active_colour;
        let them = !us;

        let from_bb = Bitboard::from(mv.from);
        let to_bb = Bitboard::from(mv.to);
        let from_to_bb = from_bb | to_bb;

        self.pieces[mv.piece] ^= from_to_bb;
        self.occupancy[us] ^= from_to_bb;

        match mv.kind {
            Quiet => (),
            Capture(piece) => {
                self.pieces[piece] ^= to_bb;
                self.halfmove_clk = 0;
                self.occupancy[them] ^= to_bb;
            }
            Promotion(piece) => {
                self.pieces[mv.piece] ^= to_bb;
                self.pieces[piece] ^= to_bb;
            }
            PromotionCapture(promo_piece, cap_piece) => {
                self.pieces[mv.piece] ^= to_bb;
                self.pieces[promo_piece] ^= to_bb;
                self.pieces[cap_piece] ^= to_bb;
                self.occupancy[them] ^= to_bb;
            }
            DoublePush => {
                self.ep_bb = (from_to_bb << 8) & (from_to_bb >> 8);
            }
            EnPassant => {
                let captured_pawn = (from_bb << 1 | from_bb >> 1) & (to_bb << 8 | to_bb >> 8);
                self.pieces[Pawn(them)] ^= captured_pawn;
                self.occupancy[them] ^= captured_pawn;
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
            }
        }

        if let King(c) = mv.piece {
            match c {
                White => self.castling -= Castling::W_KINGSIDE + Castling::W_QUEENSIDE,
                Black => self.castling -= Castling::B_KINGSIDE + Castling::B_QUEENSIDE,
            }
        }

        if let Rook(_) = mv.piece {
            match mv.from {
                Square::H1 => self.castling -= Castling::W_KINGSIDE,
                Square::A1 => self.castling -= Castling::W_QUEENSIDE,
                Square::H8 => self.castling -= Castling::B_KINGSIDE,
                Square::A8 => self.castling -= Castling::B_QUEENSIDE,
                _ => (),
            }
        }

        if let Pawn(_) = mv.piece {
            self.halfmove_clk = 0;
        }

        self.active_colour = them;

        debug_assert_eq!(self.occupied(), {
            self.gen_occupancy();
            self.occupied()
        });
    }
}
