use crate::{evaluate::evaluate, move_types::Move, position::Position};
use std::{
    fmt::Display,
    ops::Index,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Instant,
};

pub type Ply = u8;
pub type Nodes = u32;
pub type Score = i32;

const MIN_SCORE: Score = -1_000_000;
const MAX_SCORE: Score = 1_000_000;
const UNRAVEL: Score = Score::MAX;
const MAX_NODES: Nodes = Nodes::MAX;
pub const MAX_DEPTH: Ply = 64;
const PV_TBL_SIZE: usize = MAX_DEPTH as usize * (MAX_DEPTH as usize + 1);

pub struct SearchInfo {
    pub ply: Ply,
    pub searched_depth: Ply,
    pub max_depth: Ply,
    pub nodes: Nodes,
    pub max_nodes: Nodes,
    pub stop: Arc<AtomicBool>,
    pub pv: PrincipleVariation,
}

impl SearchInfo {
    pub fn new(stop: Arc<AtomicBool>) -> Self {
        SearchInfo {
            ply: 0,
            searched_depth: 0,
            max_depth: MAX_DEPTH,
            nodes: 0,
            max_nodes: MAX_NODES,
            stop,
            pv: PrincipleVariation::new(),
        }
    }
}

pub struct PrincipleVariation {
    pub moves: [Move; PV_TBL_SIZE],
    pub length: usize,
}

impl PrincipleVariation {
    fn new() -> PrincipleVariation {
        PrincipleVariation {
            moves: [Move::NULL; PV_TBL_SIZE],
            length: 0,
        }
    }

    fn index_ply(ply: u8) -> usize {
        ply as usize * (2 * MAX_DEPTH as usize + 1 - ply as usize) / 2
    }

    fn copy_down(&mut self, mv: Move, ply: u8) {
        let ply_idx = Self::index_ply(ply);
        let next_ply_idx = ply_idx + (MAX_DEPTH - ply) as usize;

        self.moves[ply_idx] = mv;
        for i in 0..(self.length - ply as usize) {
            self.moves[ply_idx + i + 1] = self.moves[next_ply_idx + i];
        }
    }
}

impl Display for PrincipleVariation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..self.length {
            write!(f, " {}", self.moves[i])?;
        }

        Ok(())
    }
}

pub fn negamax(
    pos: Position,
    mut alpha: Score,
    beta: Score,
    depth: Ply,
    info: &mut SearchInfo,
) -> Score {
    if info.nodes >= info.max_nodes || info.stop.load(Ordering::Relaxed) {
        return -UNRAVEL;
    }

    info.nodes += 1;
    if info.ply > info.searched_depth {
        info.searched_depth += 1;
    }

    if depth == 0 {
        return evaluate(&pos);
    }

    let mut best = MIN_SCORE;

    let moves = pos.gen_moves();
    for mv in moves {
        let mut next_pos = pos;
        next_pos.make_move(mv);
        if next_pos.is_check() {
            continue;
        }

        info.ply += 1;
        let score = -negamax(next_pos, -beta, -alpha, depth - 1, info);
        info.ply -= 1;

        if score == UNRAVEL {
            return -UNRAVEL;
        }

        if score > best {
            best = score;
            if score > alpha {
                alpha = score;
                info.pv.copy_down(mv, info.ply);
            }
        }

        if score >= beta {
            return best;
        }
    }

    best
}

pub fn root_search(pos: Position, mut info: SearchInfo) -> Move {
    let mut best_move = Move::NULL;
    let mut best_score = MIN_SCORE;

    let mut moves = pos.gen_moves();
    for depth in 1..=info.max_depth {
        let timer = Instant::now();
        info.nodes = 0;
        info.pv.length = depth as usize;

        moves.score(best_move);
        for (i, mv) in moves.enumerate() {
            if timer.elapsed().as_secs() > 1 {
                println!("info currmove {} currmovenumber {}", mv, i + 1)
            }

            let mut next_pos = pos;
            next_pos.make_move(mv);
            if next_pos.is_check() {
                continue;
            }

            info.ply += 1;
            let score = -negamax(next_pos, MIN_SCORE, MAX_SCORE, depth - 1, &mut info);
            info.ply -= 1;

            if score == UNRAVEL {
                return best_move;
            }

            if score > best_score {
                best_move = mv;
                best_score = score;
                info.pv.copy_down(mv, info.ply);
            }
        }

        let time = timer.elapsed();
        println!(
            "info depth {} score cp {} time {} nodes {} nps {} pv{}",
            depth,
            best_score,
            time.as_millis(),
            info.nodes,
            (1.0 / time.div_f32(info.nodes as f32).as_secs_f32()) as Nodes,
            info.pv,
        );
    }

    best_move
}
