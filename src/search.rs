use crate::{evaluate::evaluate, move_types::Move, position::Position};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

pub type Ply = u8;
pub type Nodes = u32;
pub type Score = i32;

const MIN_SCORE: Score = -1_000_000;
const MAX_SCORE: Score = 1_000_000;
const MAX_NODES: Nodes = Nodes::MAX;
pub const MAX_DEPTH: Ply = 64;

pub struct SearchInfo {
    pub curr_ply: Ply,
    pub searched_depth: Ply,
    pub max_depth: Ply,
    pub nodes: Nodes,
    pub max_nodes: Nodes,
    pub stop: Arc<AtomicBool>,
}

impl SearchInfo {
    pub fn new(stop: Arc<AtomicBool>) -> Self {
        SearchInfo {
            curr_ply: 0,
            searched_depth: 0,
            max_depth: MAX_DEPTH,
            nodes: 0,
            max_nodes: MAX_NODES,
            stop,
        }
    }
}

pub fn negamax(
    pos: Position,
    mut alpha: Score,
    beta: Score,
    depth: Ply,
    info: &mut SearchInfo,
) -> Score {
    if depth == 0 || info.nodes >= info.max_nodes || info.stop.load(Ordering::Relaxed) {
        return evaluate(&pos);
    }

    info.nodes += 1;
    if info.curr_ply > info.searched_depth {
        info.searched_depth += 1;
    }

    let mut best = MIN_SCORE;

    let moves = pos.gen_moves();
    for mv in moves {
        let mut next_pos = pos;
        next_pos.make_move(mv);
        if next_pos.is_check() {
            continue;
        }

        info.curr_ply += 1;
        let score = -negamax(next_pos, -beta, -alpha, depth - 1, info);
        info.curr_ply -= 1;

        if score > best {
            best = score;
            if score > alpha {
                alpha = score;
            }
        }

        if score >= beta {
            return best;
        }
    }

    best
}

pub fn root_search(pos: Position, mut info: SearchInfo) -> (Move, Score) {
    let mut best_move = Move::NULL;
    let mut best_score = MIN_SCORE;

    let moves = pos.gen_moves();
    for mv in moves {
        let mut next_pos = pos;
        next_pos.make_move(mv);
        if next_pos.is_check() {
            continue;
        }

        let score = -negamax(
            next_pos,
            MIN_SCORE,
            MAX_SCORE,
            info.max_depth - 1,
            &mut info,
        );

        if score > best_score {
            best_move = mv;
            best_score = score;
        }
    }

    (best_move, best_score)
}
