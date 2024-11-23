use num_enum::IntoPrimitive;
use std::ops::{Index, IndexMut};
use Colour::*;
use Piece::*;

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
            Pawn(c) => 0 + c as usize,
            Knight(c) => 2 + c as usize,
            Bishop(c) => 4 + c as usize,
            Rook(c) => 6 + c as usize,
            Queen(c) => 8 + c as usize,
            King(c) => 10 + c as usize,
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

impl TryFrom<char> for Colour {
    type Error = String;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'w' => Ok(White),
            'b' => Ok(Black),
            _ => Err(format!("Invalid char {value}")),
        }
    }
}

impl TryFrom<char> for Piece {
    type Error = String;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        let colour = if value.is_lowercase() { Black } else { White };

        match value {
            'p' | 'P' => Ok(Pawn(colour)),
            'n' | 'N' => Ok(Knight(colour)),
            'b' | 'B' => Ok(Bishop(colour)),
            'r' | 'R' => Ok(Rook(colour)),
            'q' | 'Q' => Ok(Queen(colour)),
            'k' | 'K' => Ok(King(colour)),
            _ => Err(format!("Invalid char {value}")),
        }
    }
}
