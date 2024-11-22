use num_enum::IntoPrimitive;
use std::ops::{Index, IndexMut};

#[derive(IntoPrimitive)]
#[repr(u8)]
pub enum Colour {
    White,
    Black,
}

pub enum Piece {
    Pawn(Colour),
    Knight(Colour),
    Bishop(Colour),
    Rook(Colour),
    Queen(Colour),
    King(Colour),
}

impl From<Piece> for usize {
    fn from(value: Piece) -> Self {
        match value {
            Piece::Pawn(c) => 0 + c as usize,
            Piece::Knight(c) => 2 + c as usize,
            Piece::Bishop(c) => 4 + c as usize,
            Piece::Rook(c) => 6 + c as usize,
            Piece::Queen(c) => 8 + c as usize,
            Piece::King(c) => 10 + c as usize,
        }
    }
}

impl<T> Index<Colour> for [T; 2] {
    type Output = T;

    fn index(&self, index: Colour) -> &Self::Output {
        &self[index as usize]
    }
}

impl<T> IndexMut<Colour> for [T; 2] {
    fn index_mut(&mut self, index: Colour) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

impl<T> Index<Piece> for [T; 12] {
    type Output = T;

    fn index(&self, index: Piece) -> &Self::Output {
        &self[usize::from(index)]
    }
}
impl<T> IndexMut<Piece> for [T; 12] {
    fn index_mut(&mut self, index: Piece) -> &mut Self::Output {
        &mut self[usize::from(index)]
    }
}
