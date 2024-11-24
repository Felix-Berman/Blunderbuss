use std::{
    fmt::Display,
    ops::{Add, AddAssign, Sub, SubAssign},
};

use crate::{
    bitboard::Bitboard,
    piece::{
        Colour::{self, *},
        Piece,
    },
    square::Square,
};

#[derive(Clone, Copy)]
pub struct Castling(u8);

impl Castling {
    pub const W_KINGSIDE: Castling = Castling(0b0001);
    pub const W_QUEENSIDE: Castling = Castling(0b0010);
    pub const B_KINGSIDE: Castling = Castling(0b0100);
    pub const B_QUEENSIDE: Castling = Castling(0b1000);
    pub const NONE: Castling = Castling(0);
}

impl Add for Castling {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Castling(self.0 | rhs.0)
    }
}

impl Sub for Castling {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Castling(self.0 & !rhs.0)
    }
}

impl AddAssign for Castling {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl SubAssign for Castling {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

pub struct Position {
    pieces: [Bitboard; 12],
    occupancy: [Bitboard; 2],
    active_colour: Colour,
    castling: Castling,
    ep_bb: Bitboard,
    halfmove_clk: u32,
    fullmove_clk: u32,
}

impl Position {
    pub fn new() -> Self {
        Position {
            pieces: [Bitboard::EMPTY; 12],
            occupancy: [Bitboard::EMPTY; 2],
            active_colour: White,
            castling: Castling::NONE,
            ep_bb: Bitboard::EMPTY,
            halfmove_clk: 0,
            fullmove_clk: 0,
        }
    }

    pub fn ep_sq(self) -> Option<Square> {
        self.ep_bb.bitscan()
    }

    pub fn read_fen(&mut self, fen: &str) -> Result<(), String> {
        let mut fen_iter = fen.split_whitespace();

        let fen_board = fen_iter.next().ok_or("missing board string")?;

        let mut sq = Square::A8;
        for c in fen_board.chars() {
            if c == '/' {
                continue;
            }
            if let Some(digit) = c.to_digit(10) {
                sq += digit as u8;
                continue;
            }
            if c.is_alphabetic() {
                let piece = Piece::try_from(c)?;
                self.pieces[piece].set_bit(sq);
                sq += 1;
            }
        }

        self.active_colour = Colour::try_from(
            fen_iter
                .next()
                .ok_or("missing active colour")?
                .chars()
                .next()
                .unwrap(),
        )?;

        let castling = fen_iter.next().ok_or("missing castling availability")?;
        for c in castling.chars() {
            match c {
                'K' => self.castling += Castling::W_KINGSIDE,
                'Q' => self.castling += Castling::W_QUEENSIDE,
                'k' => self.castling += Castling::B_KINGSIDE,
                'q' => self.castling += Castling::B_QUEENSIDE,
                _ => return Err(format!("Invalid char {c}")),
            }
        }

        let fen_ep = fen_iter.next().ok_or("missing en passant square")?;
        if fen_ep != "-" {
            self.ep_bb.set_bit(Square::try_from(fen_ep)?)
        }

        self.halfmove_clk = fen_iter
            .next()
            .unwrap_or("0")
            .parse::<u32>()
            .map_err(|_| "Invalid halfmove clock")?;

        self.fullmove_clk = fen_iter
            .next()
            .unwrap_or("1")
            .parse::<u32>()
            .map_err(|_| "Invalid halfmove clock")?;

        Ok(())
    }

    pub fn gen_mailbox(&self) -> Result<[Option<Piece>; 64], String> {
        let mut board: [Option<Piece>; 64] = [None; 64];
        for (piece, piece_bb) in self.pieces.iter().enumerate() {
            for sq in *piece_bb {
                board[sq] = Some(Piece::try_from(piece)?);
            }
        }

        Ok(board)
    }

    pub fn draw_board(&self) -> Result<String, String> {
        let divider = "\n  +---+---+---+---+---+---+---+---+\n";
        let board = self.gen_mailbox()?;

        let mut board_str = String::new();
        for (sq, &occupant) in board.iter().enumerate() {
            if sq % 8 == 0 {
                board_str.push_str(divider);
                board_str.push_str(&format!("{} |", 8 - sq / 8));
            }
            if let Some(piece) = occupant {
                board_str.push_str(&format!(" {piece} |"));
            } else {
                board_str.push_str("   |")
            }
        }
        board_str.push_str(divider);
        board_str.push_str("    A   B   C   D   E   F   G   H\n");

        Ok(board_str)
    }
}
