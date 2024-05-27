use crate::{bitboard::{Bitboard, Square}, movegen::{Move, MoveKind}, position::{CastlingFlags, Colour, Piece, Position}, zobrist::ZOBRIST_CODES};
use num::FromPrimitive;
use MoveKind::*;
use Colour::*;
use Square::*;
use Piece::*;

impl Position {
    pub fn make_move(&mut self, mv: Move) -> Position {

        let copy = *self;
        
        self.halfmove += 1;
        self.ply += 1;
        self.en_passant = None;
        
        let from_bb = Bitboard::from(mv.from);
        let to_bb = Bitboard::from(mv.to);
        let from_to_bb = from_bb | to_bb;
        
        self.pieces[mv.piece] ^= from_to_bb;
        self.occupancy[self.turn] ^= from_to_bb;

        self.hash ^= ZOBRIST_CODES.piece(mv.piece, mv.from);
        if let Some(sq) = self.en_passant {
            self.hash ^= ZOBRIST_CODES.en_passant(sq);
        }
        
        match mv.kind {
            Quiet => {
                self.hash ^= ZOBRIST_CODES.piece(mv.piece, mv.to);
            },
            Capture(p) => {
                self.pieces[p] ^= to_bb;
                self.halfmove = 0;
                self.last_irreversible = self.ply;
                self.occupancy[!self.turn] ^= to_bb;

                self.hash ^= ZOBRIST_CODES.piece(p, mv.to);
            },
            Promotion(p) => {
                self.pieces[mv.piece] ^= to_bb;
                self.pieces[p] ^= to_bb;

                self.hash ^= ZOBRIST_CODES.piece(p, mv.to);
            },
            PromotionCapture(p1, p2 ) => {
                self.pieces[mv.piece] ^= to_bb;
                self.pieces[p1] ^= to_bb;
                self.pieces[p2] ^= to_bb;
                self.occupancy[!self.turn] ^= to_bb;

                self.hash ^= ZOBRIST_CODES.piece(p1, mv.to) ^ ZOBRIST_CODES.piece(p2, mv.to);
            },
            DoublePawnPush => {
                self.en_passant = match self.turn {
                    White => mv.from.add(-8),
                    Black => mv.from.add(8),
                };

                self.hash ^= ZOBRIST_CODES.piece(mv.piece, mv.to) ^ ZOBRIST_CODES.en_passant(self.en_passant.unwrap());
            }
            EnPassant => {
                let captured_file = mv.to.file();
                let captured_rank = mv.from.rank();
                let captured = Square::from_i8(captured_rank*8 + captured_file).unwrap();
                self.occupancy[!self.turn].reset(captured);
                self.pieces[Pawn(!self.turn)].reset(captured);

                self.hash ^= ZOBRIST_CODES.piece(mv.piece, mv.to);
            },
            Castling(castling) => {
                let (from_to, c) = match castling {
                    CastlingFlags::WK => {
                        (Bitboard::from(F1) | Bitboard::from(H1), White)
                    },
                    CastlingFlags::WQ => {
                        (Bitboard::from(D1) | Bitboard::from(A1), White)
                    },
                    CastlingFlags::BK => {
                        (Bitboard::from(F8) | Bitboard::from(H8), Black)
                    },
                    CastlingFlags::BQ => {
                        (Bitboard::from(D8) | Bitboard::from(A8), Black)
                    },
                    _ => panic!("Attempted to castle two directions at once!"),
                };

                self.pieces[Rook(c)] ^= from_to;
                self.occupancy[c] ^= from_to;

                self.hash ^= ZOBRIST_CODES.piece(mv.piece, mv.to);
                for sq in from_to {
                    self.hash ^= ZOBRIST_CODES.piece(Rook(c), sq);
                }
                self.last_irreversible = self.ply;
            },
        }
        
        if let King(c) = mv.piece {
            match c {
                White => self.castling.remove(CastlingFlags::WK | CastlingFlags::WQ),
                Black => self.castling.remove(CastlingFlags::BK | CastlingFlags::BQ),
            }
        }
        if from_to_bb.intersects(Bitboard::from(H1)) && self.castling.contains(CastlingFlags::WK) {
            self.castling.remove(CastlingFlags::WK);
            self.last_irreversible = self.ply;
        }
        if from_to_bb.intersects(Bitboard::from(A1)) && self.castling.contains(CastlingFlags::WQ) {
            self.castling.remove(CastlingFlags::WQ);
            self.last_irreversible = self.ply;
        }
        if from_to_bb.intersects(Bitboard::from(H8)) && self.castling.contains(CastlingFlags::BK) {
            self.castling.remove(CastlingFlags::BK);
            self.last_irreversible = self.ply;
        }
        if from_to_bb.intersects(Bitboard::from(A8)) && self.castling.contains(CastlingFlags::BQ) {
            self.castling.remove(CastlingFlags::BQ);
            self.last_irreversible = self.ply;
        }
        
        if let Pawn(_) = mv.piece {
            self.halfmove = 0;
            self.last_irreversible = self.ply;
        }
        
        self.turn = !self.turn;
        copy
    }

    pub fn _unmake_move(&mut self, prev: Position) {
        *self = prev;
    }

    pub fn find_algebraic_move(&self, mv_str: &str) -> Option<Move> {
        let mut moves = self.gen_moves();
        
        moves.find(|&mv| {
            mv.to_string() == mv_str
        })
    }
}
