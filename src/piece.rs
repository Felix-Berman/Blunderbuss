use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::{
    fmt::Display,
    ops::{Index, IndexMut, Not},
};
use Colour::*;
use Piece::*;

use crate::search::Score;

#[allow(dead_code)]
pub const PIECES: [Piece; 12] = [
    Pawn(White),
    Pawn(Black),
    Knight(White),
    Knight(Black),
    Bishop(White),
    Bishop(Black),
    Rook(White),
    Rook(Black),
    Queen(White),
    Queen(Black),
    King(White),
    King(Black),
];

#[derive(PartialEq, IntoPrimitive, TryFromPrimitive, Clone, Copy, Debug)]
#[repr(u8)]
pub enum Colour {
    White,
    Black,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Piece {
    Pawn(Colour),
    Knight(Colour),
    Bishop(Colour),
    Rook(Colour),
    Queen(Colour),
    King(Colour),
}

impl Piece {
    pub fn colour(&self) -> Colour {
        match *self {
            Pawn(c) => c,
            Knight(c) => c,
            Bishop(c) => c,
            Rook(c) => c,
            Queen(c) => c,
            King(c) => c,
        }
    }

    pub fn value(&self) -> Score {
        match *self {
            Pawn(_) => 100,
            Knight(_) => 300,
            Bishop(_) => 350,
            Rook(_) => 500,
            Queen(_) => 900,
            King(_) => 0,
        }
    }
}

impl From<Piece> for usize {
    fn from(value: Piece) -> Self {
        match value {
            Pawn(c) => c as usize,
            Knight(c) => 2 + c as usize,
            Bishop(c) => 4 + c as usize,
            Rook(c) => 6 + c as usize,
            Queen(c) => 8 + c as usize,
            King(c) => 10 + c as usize,
        }
    }
}

impl From<Piece> for char {
    fn from(value: Piece) -> Self {
        match value {
            Pawn(White) => 'P',
            Pawn(Black) => 'p',
            Knight(White) => 'N',
            Knight(Black) => 'n',
            Bishop(White) => 'B',
            Bishop(Black) => 'b',
            Rook(White) => 'R',
            Rook(Black) => 'r',
            Queen(White) => 'Q',
            Queen(Black) => 'q',
            King(White) => 'K',
            King(Black) => 'k',
        }
    }
}

impl From<Colour> for char {
    fn from(value: Colour) -> Self {
        match value {
            White => 'w',
            Black => 'b',
        }
    }
}

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", char::from(*self))
    }
}

impl Display for Colour {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", char::from(*self))
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

impl<T> Index<Piece> for [T; 6] {
    type Output = T;

    fn index(&self, index: Piece) -> &Self::Output {
        &self[usize::from(index) / 2]
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

impl TryFrom<usize> for Piece {
    type Error = String;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        let colour = Colour::try_from_primitive(value as u8 % 2).unwrap();
        match value / 2 {
            0 => Ok(Pawn(colour)),
            1 => Ok(Knight(colour)),
            2 => Ok(Bishop(colour)),
            3 => Ok(Rook(colour)),
            4 => Ok(Queen(colour)),
            5 => Ok(King(colour)),
            _ => Err(format!("index ({value}) out of piece range")),
        }
    }
}

impl Not for Colour {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            White => Black,
            Black => White,
        }
    }
}
