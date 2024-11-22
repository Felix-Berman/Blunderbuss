use crate::{
    bitboard::{Bitboard, Square},
    piece::Colour,
};

pub struct Position {
    pieces: [Bitboard; 12],
    occupancy: [Bitboard; 2],
    active_colour: Colour,
    castling: u8,
    ep_bb: Bitboard,
    halfmove_clk: u32,
    fullmove_clk: u32,
}

impl Position {
    pub fn new() -> Self {
        Position {
            pieces: [Bitboard::EMPTY; 12],
            occupancy: [Bitboard::EMPTY; 2],
            active_colour: Colour::White,
            castling: 0,
            ep_bb: Bitboard::EMPTY,
            halfmove_clk: 0,
            fullmove_clk: 0,
        }
    }

    pub fn ep_sq(self) -> Square {
        self.ep_bb.bitscan()
    }
}
