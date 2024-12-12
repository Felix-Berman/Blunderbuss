use lazy_static::lazy_static;
use rand::Rng;

use crate::{
    piece::{Colour::Black, Piece},
    position::{Castling, Position},
    square::Square,
};

lazy_static! {
    pub static ref ZOBRIST_CODES: ZobristCodes = ZobristCodes::init();
}

const CASTLING_OFFSET: usize = 64 * 12;
const EN_PASSANT_OFFSET: usize = CASTLING_OFFSET + 16;
const COLOUR_OFFSET: usize = EN_PASSANT_OFFSET + 8;

pub struct ZobristCodes([u64; 793]);

impl ZobristCodes {
    pub fn init() -> Self {
        let mut codes = [0; 793];
        rand::thread_rng().fill(&mut codes[..]);
        ZobristCodes(codes)
    }

    pub fn piece(&self, piece: Piece, sq: Square) -> u64 {
        self.0[usize::from(piece) * 64 + sq as usize]
    }

    pub fn castling(&self, castling: Castling) -> u64 {
        self.0[CASTLING_OFFSET + castling.bits() as usize]
    }

    pub fn en_passant(&self, sq: Square) -> u64 {
        self.0[EN_PASSANT_OFFSET + sq.file() as usize]
    }

    pub fn colour(&self) -> u64 {
        self.0[COLOUR_OFFSET]
    }
}

impl Position {
    pub fn gen_zobrist_hash(&self) -> u64 {
        let mut hash = 0;
        for (piece, bb) in self.iter_pieces() {
            for sq in *bb {
                hash ^= ZOBRIST_CODES.piece(piece, sq);
            }
        }

        hash ^= ZOBRIST_CODES.castling(self.castling);
        //
        if let Some(sq) = self.ep_sq() {
            hash ^= ZOBRIST_CODES.en_passant(sq);
        }

        if let Black = self.active_colour {
            hash ^= ZOBRIST_CODES.colour();
        }

        hash
    }
}
