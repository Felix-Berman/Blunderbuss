use crate::{evaluate::evaluate, move_types::Move, position::Position};

const MIN: i32 = -1_000_000;
const MAX: i32 = 1_000_000;
pub const MAX_DEPTH: u8 = 64;

pub fn negamax(pos: &Position, mut alpha: i32, beta: i32, depth: u8) -> i32 {
    if depth == 0 {
        return evaluate(pos);
    }

    let mut best = MIN;

    let moves = pos.gen_moves();
    for mv in moves {
        let mut next_pos = *pos;
        next_pos.make_move(mv);

        let score = -negamax(&next_pos, -beta, -alpha, depth - 1);

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

pub fn root_search(pos: &Position, depth: u8) -> (Move, i32) {
    let mut best_move = Move::NULL;
    let mut best_score = MIN;

    let moves = pos.gen_moves();
    for mv in moves {
        let mut next_pos = *pos;
        next_pos.make_move(mv);

        let score = -negamax(&next_pos, MIN, MAX, depth - 1);

        if score > best_score {
            best_move = mv;
            best_score = score;
        }
    }

    (best_move, best_score)
}
