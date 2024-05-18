use std::{fmt::Display, ops::{Index, IndexMut, Not}};

use crate::bitboard::{Bitboard, Square::{self, *}};
use bitflags::bitflags;
use Colour::*;
use enum_iterator::Sequence;
use Piece::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Position {
    pub pieces: [Bitboard; 12],
    pub occupancy: [Bitboard; 2],
    pub turn: Colour,
    pub castling: CastlingFlags,
    pub en_passant: Option<Square>,
    pub halfmove: u8,
    pub ply: u8,
    pub hash: u64,
    pub last_irreversible: u8,
}

impl Position {

    pub fn new() -> Position {
        Position {
            pieces: [Bitboard(0); 12],
            occupancy: [Bitboard(0); 2],
            turn: White,
            castling: CastlingFlags::empty(),
            en_passant: None,
            halfmove: 0,
            ply: 0,
            hash: 0,
            last_irreversible: 0,
        }
    }

    pub fn from_fen(fen: &str) -> Position {
        let mut position = Position::new();
        position.read_fen(fen);
        position
    }

    pub fn occupied(&self) -> Bitboard {
        self.occupancy[White] | self.occupancy[Black]
    }

    pub fn piece_on(&self, sq: Square) -> Option<Piece> {
        if let Some(p) = self.pieces.iter().position(|bb| bb.is_set(sq)) {
            Some(p.into())
        } else {
            None
        }
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let delim = "\n  +---+---+---+---+---+---+---+---+\n";
        let mut square = Some(A8);
        while let Some(sq) = square {
            if sq.file() == 0 {
                write!(f, "{}", delim)?;
                write!(f, "{} |", 8 - sq.rank())?;
            }
            if let Some(p) = self.piece_on(sq) {
                write!(f, " {} |", p)?;
            } else {
                write!(f, "   |")?;
            }
            
            square = sq.add(1);
        }
        write!(f, "{}  ", delim)?;
        for file in 'a'..='h' {
            write!(f, "  {} ", file)?;
        }

        writeln!(f, "\nFen: {}", self.write_fen())
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct CastlingFlags: u8 {
        const WK = 0b0001;
        const WQ = 0b0010;
        const BK = 0b0100;
        const BQ = 0b1000;
    }
}

impl Display for CastlingFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut castling = String::new();

        if self.is_empty() {
            return write!(f, "-");
        }

        for c in self.iter() {
            match c {
                CastlingFlags::WK => castling.push('K'),
                CastlingFlags::WQ => castling.push('Q'),
                CastlingFlags::BK => castling.push('k'),
                CastlingFlags::BQ => castling.push('q'),
                _ => (),
            }
        }

        write!(f, "{}", castling)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Sequence)]
pub enum Colour {
    White,
    Black,
}

impl Display for Colour {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            White => "w",
            Black => "b",
        };
        write!(f, "{}", c)
    }
}

impl Not for Colour {
    type Output = Colour;
    fn not(self) -> Self::Output {
        match self {
            White => Black,
            Black => White,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Sequence)]
pub enum Piece {
    Pawn(Colour),
    Knight(Colour),
    Bishop(Colour),
    Rook(Colour),
    Queen(Colour),
    King(Colour),
}

impl Piece {
    pub fn iter_colour(c: Colour) -> impl Iterator<Item = Piece> {
        let pieces = [Pawn(c), Knight(c), Bishop(c), Rook(c), Queen(c), King(c)];
        pieces.into_iter()
    }

}

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let p = match self {
            Pawn(White) => "P",
            Knight(White) => "N",
            Bishop(White) => "B",
            Rook(White) => "R",
            Queen(White) => "Q",
            King(White) => "K",
            Pawn(Black) => "p",
            Knight(Black) => "n",
            Bishop(Black) => "b",
            Rook(Black) => "r",
            Queen(Black) => "q",
            King(Black) => "k",
        };
        write!(f, "{}", p)
    }
}

impl From<Piece> for usize {
    fn from(piece: Piece) -> Self {
        match piece {
            Pawn(White) => 0,
            Knight(White) => 1,
            Bishop(White) => 2,
            Rook(White) => 3,
            Queen(White) => 4,
            King(White) => 5,
            Pawn(Black) => 6,
            Knight(Black) => 7,
            Bishop(Black) => 8,
            Rook(Black) => 9,
            Queen(Black) => 10,
            King(Black) => 11,
        }
    }
}

impl From<usize> for Piece {
    fn from(idx: usize) -> Self {
        match idx {
            0 => Pawn(White),
            1 => Knight(White),
            2 => Bishop(White),
            3 => Rook(White),
            4 => Queen(White),
            5 => King(White),
            6 => Pawn(Black),
            7 => Knight(Black),
            8 => Bishop(Black),
            9 => Rook(Black),
            10 => Queen(Black),
            11 => King(Black),
            _ => unreachable!(),
        }
    }
}

impl Index<Piece> for [Bitboard; 12] {  
    type Output = Bitboard;
    fn index(&self, piece: Piece) -> &Self::Output {
        match piece {
            Pawn(White) => &self[0],
            Knight(White) => &self[1],
            Bishop(White) => &self[2],
            Rook(White) => &self[3],
            Queen(White) => &self[4],
            King(White) => &self[5],
            Pawn(Black) => &self[6],
            Knight(Black) => &self[7],
            Bishop(Black) => &self[8],
            Rook(Black) => &self[9],
            Queen(Black) => &self[10],
            King(Black) => &self[11],
        }
    }
}

impl IndexMut<Piece> for [Bitboard; 12] {
    fn index_mut(&mut self, piece: Piece) -> &mut Self::Output {
        match piece {
            Pawn(White) => &mut self[0],
            Knight(White) => &mut self[1],
            Bishop(White) => &mut self[2],
            Rook(White) => &mut self[3],
            Queen(White) => &mut self[4],
            King(White) => &mut self[5],
            Pawn(Black) => &mut self[6],
            Knight(Black) => &mut self[7],
            Bishop(Black) => &mut self[8],
            Rook(Black) => &mut self[9],
            Queen(Black) => &mut self[10],
            King(Black) => &mut self[11],
        }
    }
}

impl<T> Index<Colour> for [T; 12] {
    type Output = [T];
    fn index(&self, colour: Colour) -> &Self::Output {
        match colour {
            White => &self[0..6],
            Black => &self[6..12],
        }
    }
}

impl<T> IndexMut<Colour> for [T; 12] {
    fn index_mut(&mut self, colour: Colour) -> &mut Self::Output {
        match colour {
            White => &mut self[0..6],
            Black => &mut self[6..12],
        }
    }
}

impl<T> Index<Colour> for [T; 2] {
    type Output = T;
    fn index(&self, colour: Colour) -> &Self::Output {
        match colour {
            White => &self[0],
            Black => &self[1],
        }
    }
}

impl<T> IndexMut<Colour> for [T; 2] {
    fn index_mut(&mut self, colour: Colour) -> &mut Self::Output {
        match colour {
            White => &mut self[0],
            Black => &mut self[1],
        }
    }
}