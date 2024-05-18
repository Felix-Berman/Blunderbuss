use std::num::ParseIntError;

use num::FromPrimitive;

use crate::bitboard::{Bitboard, Square};
use crate::position::{CastlingFlags, Piece, Position};
use crate::position::{Piece::*, Colour::*};

const N_FIELDS: usize = 6;
pub const STARTING_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

type FenResult = Result<(), FenError>;
type Parser = fn(&mut Position, &str) -> FenResult;

#[derive(Debug)]
enum FenError {
    InvalidBoardSize,
    InvalidBoardChar(char),
    Turn(String),
    Castling(char),
    EnPassant(String),
    HalfMove(ParseIntError),
    FullMove(ParseIntError),
}

impl std::fmt::Display for FenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FenError::InvalidBoardSize => write!(f, "Invalid board size"),
            FenError::InvalidBoardChar(c) => write!(f, "Invalid character in board: {}", c),
            FenError::Turn(s) => write!(f, "Invalid turn: {}", s),
            FenError::Castling(c) => write!(f, "Invalid castling: {}", c),
            FenError::EnPassant(s) => write!(f, "Invalid en passant: {}", s),
            FenError::HalfMove(e) => write!(f, "Invalid halfmove: {}", e),
            FenError::FullMove(e) => write!(f, "Invalid fullmove: {}", e),
        }
    }
}

impl Position {
    pub fn read_fen(&mut self, fen: &str) {
        *self = Position::new();
        let fen = fen.replace('\"', "");
        let fields: Vec<&str> = fen.split_whitespace().collect();

        let parsers: [Parser; N_FIELDS] = [board, turn, castling, ep, halfmove, fullmove];

        fields.iter().enumerate().for_each(|(i, x)| {
            match parsers[i](self, x) {
                Ok(_) => (),
                Err(e) => {
                    println!("info string Error parsing FEN: {}", e);
                }
            }
        });
        
        self.occupancy[White] = self.pieces[White].iter().fold(Bitboard(0), |acc, x| acc | *x);
        self.occupancy[Black] = self.pieces[Black].iter().fold(Bitboard(0), |acc, x| acc | *x);
        self.gen_zobrist_hash();
    }

    pub fn write_fen(&self) -> String {
        let mut fen = String::new();

        // board
        for r in 0..8 {
            let mut empty_count: u8 = 0;
            for f in 0..8 {
                let sq = Square::from_u8(r*8 + f).unwrap();
                if let Some(p) = enum_iterator::all::<Piece>().find(|p| self.pieces[*p].is_set(sq)) {
                    if empty_count != 0 {
                        fen.push((b'0' + empty_count) as char);
                    }
                    fen.push_str(&p.to_string());
                    empty_count = 0;
                } else {
                    empty_count += 1;
                }
            }

            if empty_count != 0 {
                fen.push((b'0' + empty_count) as char);
            }
            if r < 7 { 
                fen.push('/') 
            };
        }

        fen.push(' ');
        match self.turn {
            White => fen.push('w'),
            Black => fen.push('b'),
        }

        fen.push_str(&format!(" {}", self.castling));

        if let Some(sq) = self.en_passant {
            fen.push_str(&format!(" {}", sq));
        } else {
            fen.push_str(" -");
        }

        fen.push_str(&format!(" {}", self.halfmove));

        let fullmove = match self.turn {
            White => (self.ply + 2) / 2,
            Black => (self.ply + 1) / 2,
        };
        fen.push_str(&format!(" {}", fullmove));

        fen
    }
}

fn board(position: &mut Position, board: &str) -> FenResult {
    let mut i = 0;
    let mut j = 0;
    for char in board.chars() {
        let sq = Square::A8.add(8*j + i).ok_or(FenError::InvalidBoardSize)?;
        i+= 1;
        match char {
            'K' => position.pieces[King(White)].set(sq),
            'Q' => position.pieces[Queen(White)].set(sq),
            'R' => position.pieces[Rook(White)].set(sq),
            'B' => position.pieces[Bishop(White)].set(sq),
            'N' => position.pieces[Knight(White)].set(sq),
            'P' => position.pieces[Pawn(White)].set(sq),
            'k' => position.pieces[King(Black)].set(sq),
            'q' => position.pieces[Queen(Black)].set(sq),
            'r' => position.pieces[Rook(Black)].set(sq),
            'b' => position.pieces[Bishop(Black)].set(sq),
            'n' => position.pieces[Knight(Black)].set(sq),
            'p' => position.pieces[Pawn(Black)].set(sq),
            '1'..='7' => i += char.to_digit(10).unwrap() as i8 - 1,
            '8' => (),
            '/' => {
                j += 1;
                i = 0
            },
            _=> return Err(FenError::InvalidBoardChar(char))
        }
    }

Ok(())
}

fn turn(position: &mut Position, turn: &str) -> FenResult {
    position.turn = match turn {
        "w" => White,
        "b" => Black,
        _=> return Err(FenError::Turn(turn.to_string()))
    };

    Ok(())
}

fn castling(position: &mut Position, castling: &str) -> FenResult {
    for char in castling.chars() {
        position.castling |= match char {
            'K' => CastlingFlags::WK,
            'k' => CastlingFlags::BK,
            'Q' => CastlingFlags::WQ,
            'q' => CastlingFlags::BQ,
            '-' => CastlingFlags::empty(),
            _=> return Err(FenError::Castling(char))
        }
    }

    Ok(())
}

fn ep(position: &mut Position, ep: &str) -> FenResult {
    position.en_passant = match ep {
        "-" => None,
        _=> Some(Square::from_algebraic(ep).ok_or(FenError::EnPassant(ep.to_string()))?)
    };

    Ok(())
}

fn halfmove(position: &mut Position, halfmove: &str) -> FenResult {
    position.halfmove = match halfmove.parse() {
        Ok(x) => x,
        Err(e) => return Err(FenError::HalfMove(e))
    };

    Ok(())
}

fn fullmove(position: &mut Position, fullmove: &str) -> FenResult {
    let fullmove: u8 = match fullmove.parse() {
        Ok(x) => x,
        Err(e) => return Err(FenError::FullMove(e))
    };

    match position.turn {
        White => position.ply = fullmove * 2 - 2,
        Black => position.ply = fullmove * 2 - 1,
    }

    Ok(())
}