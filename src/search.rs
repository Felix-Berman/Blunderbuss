use crate::{
    bitboard::Bitboard,
    evaluate::evaluate,
    move_types::{Move, MoveKind::*, MoveList},
    movegen::{bishop_attacks, rook_attacks},
    piece::Piece::*,
    position::Position,
};
use std::{
    fmt::Display,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

pub type Ply = u8;
pub type Nodes = u32;
pub type Score = i32;

const MIN_SCORE: Score = -CHECKMATE;
const MAX_SCORE: Score = CHECKMATE;
const UNRAVEL: Score = Score::MAX;
const CHECKMATE: Score = UNRAVEL - 1;
const STALEMATE: Score = 0;
const MAX_NODES: Nodes = Nodes::MAX;
pub const MAX_DEPTH: Ply = 64;
const PV_TBL_SIZE: usize = MAX_DEPTH as usize * (MAX_DEPTH as usize + 1);
const ACCEPTABLE_BEST_MOVE_VARATION: f32 = 0.25;
const ACCEPTABLE_SCORE_STABILITY: f32 = 0.8;

pub struct SearchInfo {
    pub ply: Ply,
    pub searched_depth: Ply,
    pub max_depth: Ply,
    pub nodes: Nodes,
    pub max_nodes: Nodes,
    pub stop: Arc<AtomicBool>,
    pub pv: PrincipleVariation,
    pub allowed_time: Duration,
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
            allowed_time: Duration::ZERO,
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

    fn reset_ply(&mut self, ply: u8) {
        self.moves[Self::index_ply(ply)] = Move::NULL;
    }

    fn copy_down(&mut self, mv: Move, ply: u8) {
        let ply_idx = Self::index_ply(ply);
        let next_ply_idx = ply_idx + (MAX_DEPTH - ply) as usize;

        self.moves[ply_idx] = mv;
        for i in 0..(self.length - ply as usize) {
            if self.moves[next_ply_idx + i] == Move::NULL {
                break;
            }
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
        return quiescence_search(pos, alpha, beta, info);
    }

    let mut best = -CHECKMATE + info.ply as i32;
    let mut legal_move_count = 0;
    info.pv.reset_ply(info.ply);
    let moves = pos.gen_moves();
    for mv in moves {
        let mut next_pos = pos;
        next_pos.make_move(mv);
        if next_pos.is_check(!next_pos.active_colour) {
            continue;
        }
        legal_move_count += 1;

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

    if legal_move_count == 0 {
        if pos.is_check(pos.active_colour) {
            return -CHECKMATE + info.ply as i32;
        } else {
            return STALEMATE;
        }
    }

    best
}

pub fn root_search(pos: Position, mut info: SearchInfo) -> Move {
    let mut best_move = Move::NULL;
    let mut iteration_moves = [Move::NULL; MAX_DEPTH as usize];
    let mut iteration_scores = [0; MAX_DEPTH as usize];
    let outer_timer = Instant::now();

    let mut moves = pos.gen_moves();
    for depth in 1..=info.max_depth {
        if !info.allowed_time.is_zero() && outer_timer.elapsed() > info.allowed_time / 2 {
            break;
        }

        let inner_timer = Instant::now();
        info.nodes = 0;
        info.pv.length = depth as usize;

        let mut best_score = -CHECKMATE;

        let mut legal_move_count = 0;
        moves.score(best_move);
        for (i, mv) in moves.enumerate() {
            if outer_timer.elapsed().as_secs() > 1 {
                println!("info currmove {} currmovenumber {}", mv, i + 1);
            }

            let mut next_pos = pos;
            next_pos.make_move(mv);
            if next_pos.is_check(!next_pos.active_colour) {
                continue;
            }
            legal_move_count += 1;

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

        let time = inner_timer.elapsed();
        let mate_in = CHECKMATE - best_score.abs();
        let score = if mate_in <= depth as i32 {
            format!("mate {}", best_score.signum() * (mate_in + 1) / 2)
        } else {
            format!("cp {}", best_score)
        };
        println!(
            "info depth {} seldepth {} score {} time {} nodes {} nps {} pv{}",
            depth,
            info.searched_depth,
            score,
            time.as_millis(),
            info.nodes,
            (1.0 / time.div_f32(info.nodes as f32).as_secs_f32()) as Nodes,
            info.pv,
        );

        if legal_move_count <= 1 {
            break;
        }

        iteration_moves[depth as usize - 1] = best_move;
        iteration_scores[depth as usize - 1] = best_score;

        // todo: break on fluctations between equally good moves (requires multi-pv)
        let mut prev_move = Move::NULL;
        let mut best_move_variations = 0.0;
        let mut prev_score = 0;
        for (moves_checked, (mv, score)) in iteration_moves
            .iter()
            .zip(iteration_scores.iter())
            .rev()
            .enumerate()
        {
            if *mv != prev_move {
                best_move_variations += 1.0;
            }

            if best_move_variations / (moves_checked as f32 + 1.0) < ACCEPTABLE_BEST_MOVE_VARATION
                && *score as f32 / prev_score as f32 > ACCEPTABLE_SCORE_STABILITY
            {
                break;
            }

            prev_move = *mv;
            prev_score = *score;
        }
    }

    best_move
}

fn quiescence_search(pos: Position, mut alpha: i32, beta: i32, info: &mut SearchInfo) -> Score {
    let standing_pat = evaluate(&pos);

    if standing_pat >= beta {
        return beta;
    }
    if alpha < standing_pat {
        alpha = standing_pat;
    }

    info.nodes += 1;
    if info.ply > info.searched_depth {
        info.searched_depth = info.ply;
    }

    let mut moves = MoveList::new();
    pos.gen_captures(&mut moves);
    for mv in moves {
        if !swap_off(&pos, mv) {
            continue;
        }
        let mut next_pos = pos;
        next_pos.make_move(mv);
        if next_pos.is_check(!next_pos.active_colour) {
            continue;
        }

        info.ply += 1;
        let score = -quiescence_search(next_pos, -beta, -alpha, info);
        info.ply -= 1;

        if score >= beta {
            return beta;
        }

        if score > alpha {
            alpha = score;
        }
    }

    alpha
}

// todo: check for pins
// returns true if the material exchange initiated by a given capture is favourable or equal based
// on piece value alone
fn swap_off(pos: &Position, mv: Move) -> bool {
    let attacker = pos.piece_on(mv.from).unwrap();
    let target = match mv.kind {
        Capture(p) => p,
        PromotionCapture(_, p) => p,
        EnPassant => return true,
        _ => panic!("swap off should only consider captures"),
    };

    let mut swap_score = target.value() - attacker.value();
    let mut can_take = true;
    // can take a winning or even exchange
    if swap_score >= 0 {
        return can_take;
    }

    let mut occupied = pos.occupied() ^ Bitboard::from(mv.from);
    let mut attackers_defenders = pos.square_attackers(mv.to, occupied);

    // players alternately take with least valueable piece. If their swap score is positive they
    // don't need to make any recapture to come out on top and can stop (break).
    let mut turn = pos.active_colour;
    loop {
        // even trade can be taken
        if swap_score == 0 {
            return true;
        }

        turn = !turn;

        attackers_defenders &= occupied;
        let attackers = attackers_defenders & pos.occupancy[turn];

        if attackers.is_empty() {
            break;
        }

        can_take = !can_take;
        swap_score = -swap_score;

        // check for attackers in value order
        if let Some(sq) = (attackers & pos.pieces[Pawn(turn)]).bitscan() {
            swap_score -= Pawn(turn).value();
            occupied ^= Bitboard::from(sq);

            attackers_defenders |=
                bishop_attacks(sq, occupied) & (pos.pieces[Bishop(turn)] | pos.pieces[Queen(turn)]);
        } else if let Some(sq) = (attackers & pos.pieces[Knight(turn)]).bitscan() {
            swap_score -= Knight(turn).value();
            occupied ^= Bitboard::from(sq);
        } else if let Some(sq) = (attackers & pos.pieces[Bishop(turn)]).bitscan() {
            swap_score -= Bishop(turn).value();
            occupied ^= Bitboard::from(sq);

            attackers_defenders |=
                bishop_attacks(sq, occupied) & (pos.pieces[Bishop(turn)] | pos.pieces[Queen(turn)]);
        } else if let Some(sq) = (attackers & pos.pieces[Rook(turn)]).bitscan() {
            swap_score -= Rook(turn).value();
            occupied ^= Bitboard::from(sq);

            attackers_defenders |=
                rook_attacks(sq, occupied) & (pos.pieces[Rook(turn)] | pos.pieces[Queen(turn)]);
        } else if let Some(sq) = (attackers & pos.pieces[Queen(turn)]).bitscan() {
            swap_score -= Queen(turn).value();
            occupied ^= Bitboard::from(sq);

            attackers_defenders |=
                rook_attacks(sq, occupied) & (pos.pieces[Rook(turn)] | pos.pieces[Queen(turn)]);
            attackers_defenders |=
                bishop_attacks(sq, occupied) & (pos.pieces[Bishop(turn)] | pos.pieces[Queen(turn)]);
        } else {
            // if opponent has any remaining attackers, cannot take with king
            if attackers_defenders.intersects(&pos.occupancy[!turn]) {
                can_take = !can_take;
            }

            break;
        }

        if swap_score.is_positive() {
            break;
        }
    }

    can_take
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swap_test_trade() {
        let mut pos = Position::new();
        _ = pos.read_fen("8/8/2p5/3p4/4P3/8/8/8 w - - 0 1");
        let mv = pos.find_algebraic_move("e4d5").unwrap();
        assert!(swap_off(&pos, mv));

        pos.flip();
        let mv = pos.find_algebraic_move("e5d4").unwrap();
        assert!(swap_off(&pos, mv));
    }

    #[test]
    fn swap_test_loss() {
        let mut pos = Position::new();
        _ = pos.read_fen("8/4n3/8/3p4/8/4N3/8/8 w - - 0 1");
        let mv = pos.find_algebraic_move("e3d5").unwrap();
        assert!(!swap_off(&pos, mv));

        pos.flip();
        let mv = pos.find_algebraic_move("e6d4").unwrap();
        assert!(!swap_off(&pos, mv));
    }

    #[test]
    fn swap_test_win() {
        let mut pos = Position::new();
        _ = pos.read_fen("8/4n3/8/3p4/8/4N3/8/3R4 w - - 0 1");
        let mv = pos.find_algebraic_move("e3d5").unwrap();
        assert!(swap_off(&pos, mv));

        pos.flip();
        let mv = pos.find_algebraic_move("e6d4").unwrap();
        assert!(swap_off(&pos, mv));
    }

    #[test]
    fn swap_test_xray() {
        let mut pos = Position::new();
        _ = pos.read_fen("1k1r3q/1ppn3p/p4b2/4p3/8/P2N2P1/1PP1R1BP/2K1Q3 w - - 0 1");
        let mv = pos.find_algebraic_move("d3e5").unwrap();
        assert!(!swap_off(&pos, mv));

        pos.flip();
        let mv = pos.find_algebraic_move("d6e4").unwrap();
        assert!(!swap_off(&pos, mv));
    }

    #[test]
    fn swap_test_king() {
        let mut pos = Position::new();
        _ = pos.read_fen("3q4/3r4/8/2Kp4/8/8/3R4/3Q4 w - - 0 1");
        let mv = pos.find_algebraic_move("d2d5").unwrap();
        println!("{}", pos.draw_board());
        assert!(swap_off(&pos, mv));

        pos.flip();
        let mv = pos.find_algebraic_move("d7d4").unwrap();
        assert!(swap_off(&pos, mv));

        _ = pos.read_fen("3q4/3r4/8/2Kpk3/8/8/3R4/3Q4 w - - 0 1");
        let mv = pos.find_algebraic_move("d2d5").unwrap();
        assert!(!swap_off(&pos, mv));

        pos.flip();
        let mv = pos.find_algebraic_move("d7d4").unwrap();
        assert!(!swap_off(&pos, mv));
    }
}
