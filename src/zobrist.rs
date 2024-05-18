use rand::Rng;
use lazy_static::lazy_static;
use crate::{bitboard::Square, position::{CastlingFlags, Colour::*, Piece::{self, *}, Position}};

lazy_static! {
    pub static ref ZOBRIST_CODES: ZobristCodes = ZobristCodes::init();
}

const PAWN_OFFSET: usize = 64*10; // 10 pieces with 64 squares
const CASTLING_OFFSET: usize = PAWN_OFFSET + 48*2; // 2 pawns with 48 squares (excluding back ranks)
const EN_PASSANT_OFFSET: usize = CASTLING_OFFSET + 16; // 2^4 castling arrangements
const TURN_OFFSET: usize = EN_PASSANT_OFFSET + 8; // 8 possible en-passent files

pub struct ZobristCodes([u64; 761]);

impl ZobristCodes {
    pub fn init() -> Self {
        let mut codes = [0; 761];
        rand::thread_rng().fill(&mut codes[..]);
        ZobristCodes(codes)
    }

    pub fn piece(&self, piece: Piece, sq: Square) -> u64 {
        match piece {
            Pawn(White) => self.0[PAWN_OFFSET + sq as usize - 8],
            Pawn(Black) => self.0[PAWN_OFFSET + 48 + sq as usize - 8],
            _ => self.0[usize::from(piece) * 64 + sq as usize],
        }
    }

    pub fn castling(&self, castling: CastlingFlags) -> u64 {
        self.0[CASTLING_OFFSET + castling.bits() as usize]
    }

    pub fn en_passant(&self, sq: Square) -> u64 {
        self.0[EN_PASSANT_OFFSET + sq.file() as usize]
    }

    pub fn turn(&self) -> u64 {
        self.0[TURN_OFFSET]
    }
}


impl Position {
    pub fn gen_zobrist_hash(&mut self) {
        self.hash = 0;

        for (piece, bb) in self.pieces.iter().enumerate() {
            for sq in *bb {
                self.hash ^= ZOBRIST_CODES.piece(piece.into(), sq);
            }
        }

        self.hash ^= ZOBRIST_CODES.castling(self.castling);

        if let Some(sq) = self.en_passant {
            self.hash ^= ZOBRIST_CODES.en_passant(sq);
        }

        if let White = self.turn {
            self.hash ^= ZOBRIST_CODES.turn();
        }
    }
}