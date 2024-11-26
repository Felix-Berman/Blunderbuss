const MAX_MOVES: usize = 256;

use std::fmt::Display;

use crate::{
    piece::{
        Colour::*,
        Piece::{self, *},
    },
    position::Castling,
    square::Square::{self, *},
};

#[derive(Clone, Copy)]
pub enum MoveKind {
    Quiet,
    Capture(Piece),
    DoublePush,
    EnPassant,
    Castling(Castling),
    Promotion(Piece),
    PromotionCapture(Piece, Piece),
}

#[derive(Clone, Copy)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub piece: Piece,
    pub kind: MoveKind,
}

impl Move {
    pub const NULL: Move = Move::new();

    pub const fn new() -> Move {
        Move {
            from: A1,
            to: A1,
            piece: Pawn(White),
            kind: MoveKind::Quiet,
        }
    }
}

#[derive(Clone, Copy)]
pub struct MoveList {
    pub moves: [Move; MAX_MOVES],
    pub sort_scores: [u8; MAX_MOVES],
    pub length: usize,
    pub curr: usize,
}

impl MoveList {
    pub fn new() -> MoveList {
        MoveList {
            moves: [Move::NULL; MAX_MOVES],
            sort_scores: [0; MAX_MOVES],
            length: 0,
            curr: 0,
        }
    }

    pub fn push(&mut self, mv: Move) {
        self.moves[self.length] = mv;
        self.length += 1;
    }

    pub fn pop(&mut self) -> Move {
        self.length -= 1;
        self.moves[self.length]
    }
}

impl Iterator for MoveList {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr >= self.length {
            return None;
        }

        let mut next_best = self.moves[self.curr];
        let mut best_score = self.sort_scores[self.curr];

        for i in (self.curr + 1)..self.length {
            if self.sort_scores[i] > best_score {
                next_best = self.moves[i];
                best_score = self.sort_scores[i];
                self.moves[i] = self.moves[self.curr];
                self.sort_scores[i] = self.sort_scores[self.curr];
                self.moves[self.curr] = next_best;
                self.sort_scores[self.curr] = best_score;
            }
        }

        self.curr += 1;
        Some(next_best)
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut str = format!("{}{}", self.from, self.to);
        if let MoveKind::Promotion(piece) | MoveKind::PromotionCapture(piece, _) = self.kind {
            str.push(piece.into())
        }

        write!(f, "{}", str)
    }
}
