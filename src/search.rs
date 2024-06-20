use std::{
    cmp::{max, min},
    time::Instant,
};

use crossbeam_channel::{Receiver, SendError, Sender};

use crate::{
    bitboard::{Bitboard, Square},
    engine::MAX_GAME_PLY,
    eval::evaluate,
    magic::{bishop_attacks, rook_attacks},
    movegen::{knight_attacks, pawn_attacks, Move, MoveKind, MoveList},
    position::{
        Colour::*,
        Piece::{self, *},
        Position,
    },
};

pub const MAX_DEPTH: usize = 64;
const PV_SIZE: usize = MAX_DEPTH * (MAX_DEPTH + 1) / 2;
const STALEMATE: i32 = 0;
pub const CHECKMATE: i32 = 1_000_000;
const UNRAVEL: i32 = CHECKMATE + 1;
const HALFMOVE_DRAW_COUNT: u8 = 100;

type SendResult = Result<(), SendError<SendInfo>>;

#[derive(Debug)]
pub enum SearchCommand {
    Stop,
    _PonderHit,
}

#[derive(Debug)]
pub enum SendInfo {
    Full(FullInfo),
    CurrMove(CurrMoveInfo),
    Done(Option<Move>),
}

#[derive(Debug)]
pub struct FullInfo {
    pub depth: u8,
    pub seldepth: u8,
    pub score: i32,
    pub nodes: u32,
    pub time: u32,
    pub pv: [Option<Move>; MAX_DEPTH],
}

#[derive(Debug)]
pub struct CurrMoveInfo {
    pub depth: u8,
    pub mv: Move,
    pub mv_num: u8,
    pub time: u32,
}

#[derive(Debug)]
pub struct SearchInfo {
    pub depth: u8,
    pub seldepth: u8,
    pub score: i32,
    pub nodes: u32,
    pub stop_nodes: u32,
    pub time: Instant,
    pub triangular_pv: [Option<Move>; PV_SIZE],
    pub current_branch: [Option<Move>; MAX_DEPTH],
    pub history: [u64; MAX_GAME_PLY],
    pub tx: Sender<SendInfo>,
    pub rx: Receiver<SearchCommand>,
    pub stop: bool,
}

impl SearchInfo {
    fn new(
        stop_nodes: u32,
        history: [u64; MAX_GAME_PLY],
        tx: Sender<SendInfo>,
        rx: Receiver<SearchCommand>,
    ) -> Self {
        SearchInfo {
            depth: 0,
            seldepth: 0,
            score: 0,
            nodes: 0,
            stop_nodes,
            time: Instant::now(),
            triangular_pv: [None; PV_SIZE],
            current_branch: [None; MAX_DEPTH],
            history,
            tx,
            rx,
            stop: false,
        }
    }

    fn send_full(&mut self) -> SendResult {
        let full = FullInfo {
            depth: self.depth,
            seldepth: self.seldepth,
            score: self.score,
            nodes: self.nodes,
            time: self.time.elapsed().as_millis() as u32,
            pv: self.triangular_pv[0..MAX_DEPTH].try_into().unwrap(),
        };

        self.tx.send(SendInfo::Full(full))
    }

    fn send_currmove(&self, mv: Move, mv_num: u8) -> SendResult {
        let curr_move = CurrMoveInfo {
            depth: self.depth,
            mv,
            mv_num,
            time: self.time.elapsed().as_millis() as u32,
        };

        self.tx.send(SendInfo::CurrMove(curr_move))
    }

    fn send_bestmove(&self) -> SendResult {
        self.tx.send(SendInfo::Done(self.triangular_pv[0]))
    }

    fn hoist_pv(&mut self, target: usize, source: usize, len: usize) {
        for i in 0..len {
            let Some(mv) = self.triangular_pv[source + i] else {
                break;
            };
            self.triangular_pv[target + i] = Some(mv);
        }
    }
}

pub fn iterative_deepening(
    mut pos: Position,
    stop_depth: u8,
    stop_nodes: u32,
    history: [u64; MAX_GAME_PLY],
    tx: Sender<SendInfo>,
    rx: Receiver<SearchCommand>,
) {
    // clear receiver in case stop sent from previous search
    for _ in rx.try_iter() {
        print!("");
    }

    let mut info = SearchInfo::new(stop_nodes, history, tx, rx);

    for depth in 1..=stop_depth {
        info.time = Instant::now();
        info.depth = depth;
        info.nodes = 0;
        info.score = negamax(&mut pos, -i32::MAX, i32::MAX, depth, 0, 0, &mut info);

        if info.score < UNRAVEL {
            info.send_full().unwrap();
        }

        if CHECKMATE - info.score.abs() <= depth as i32 {
            break;
        }

        if info.stop || info.nodes >= info.stop_nodes {
            break;
        }
    }

    info.send_bestmove().unwrap();
}

fn negamax(
    pos: &mut Position,
    mut alpha: i32,
    beta: i32,
    depth: u8,
    ply: usize,
    pv_idx: usize,
    info: &mut SearchInfo,
) -> i32 {
    if info.depth > 1 && info.nodes % 10_000 == 0 && info.rx.try_recv().is_ok() {
        info.stop = true;
        return UNRAVEL;
    }

    if ply as u8 > info.seldepth {
        info.seldepth = ply as u8;
    }

    if pos.halfmove >= HALFMOVE_DRAW_COUNT || detect_repetition(pos, info.history, ply < 2) {
        return STALEMATE;
    }

    if depth == 0 {
        return quiescence_search(pos, alpha, beta, ply, info);
    }

    let next_pv_idx = pv_idx + MAX_DEPTH - ply;

    let mut moves = pos.gen_moves();
    moves.score(ply, info);

    let mut legal_moves = 0;
    for mv in moves {
        let prev = pos.make_move(mv);
        if pos.is_check(prev.turn) {
            *pos = prev;
            continue;
        }

        info.current_branch[ply] = Some(mv);
        info.history[prev.ply as usize] = prev.hash;
        legal_moves += 1;

        if ply == 0 {
            _ = info.send_currmove(mv, legal_moves);
        }

        let score = -negamax(pos, -beta, -alpha, depth - 1, ply + 1, next_pv_idx, info);

        if info.nodes > info.stop_nodes || info.stop {
            return min(alpha.abs(), UNRAVEL);
        }

        info.nodes += 1;

        if score >= beta {
            return beta;
        }

        if score > alpha {
            alpha = score;
            info.triangular_pv[pv_idx] = Some(mv);
            info.hoist_pv(pv_idx + 1, next_pv_idx, MAX_DEPTH - ply - 1);
        }

        *pos = prev;
    }

    if legal_moves == 0 {
        if pos.is_check(pos.turn) {
            return -CHECKMATE + ply as i32;
        } else {
            return STALEMATE;
        }
    }

    alpha
}

fn quiescence_search(
    pos: &mut Position,
    mut alpha: i32,
    beta: i32,
    ply: usize,
    info: &mut SearchInfo,
) -> i32 {
    if info.depth > 1 && info.nodes % 10_000 == 0 && info.rx.try_recv().is_ok() {
        info.stop = true;
        return UNRAVEL;
    }

    if ply as u8 > info.seldepth {
        info.seldepth = ply as u8;
    }

    let standing_pat = evaluate(pos);
    if standing_pat >= beta {
        return beta;
    }

    if standing_pat > alpha {
        alpha = standing_pat;
    }

    let mut captures = MoveList::new();
    pos.gen_captures(&mut captures);
    captures.score(ply, info);

    for capture in captures {
        let target = match capture.kind {
            MoveKind::Capture(target) | MoveKind::PromotionCapture(_, target) => target,
            MoveKind::EnPassant => Pawn(!pos.turn),
            _ => panic!(),
        };

        if static_exchange_evaluation(pos, capture.from, capture.to, capture.piece, target) < 0 {
            continue;
        }

        // delta pruning. Need to consider effect on endgame
        if let MoveKind::Capture(piece) | MoveKind::PromotionCapture(_, piece) = capture.kind {
            if standing_pat + piece.value() + 200 <= alpha {
                continue;
            }
        }

        let prev = pos.make_move(capture);
        if pos.is_check(prev.turn) {
            *pos = prev;
            continue;
        }

        info.nodes += 1;
        let score = -quiescence_search(pos, -beta, -alpha, ply + 1, info);
        *pos = prev;

        if info.nodes > info.stop_nodes || info.stop {
            return min(alpha.abs(), UNRAVEL);
        }

        if score >= beta {
            return beta;
        }

        if score > alpha {
            alpha = score;
        }
    }

    alpha
}

fn static_exchange_evaluation(
    position: &Position,
    from: Square,
    to: Square,
    mut attacker: Piece,
    target: Piece,
) -> i32 {
    let mut gain = [0; 32];
    let mut depth = 0;
    let mut side = position.turn;

    let pawns = position.pieces[Pawn(White)] | position.pieces[Pawn(Black)];
    let knights = position.pieces[Knight(White)] | position.pieces[Knight(Black)];
    let bishops = position.pieces[Bishop(White)] | position.pieces[Bishop(Black)];
    let rooks = position.pieces[Rook(White)] | position.pieces[Rook(Black)];
    let queens = position.pieces[Queen(White)] | position.pieces[Queen(Black)];
    let may_xray = pawns | bishops | rooks | queens;

    let mut from_bb = Bitboard::from(from);
    let mut occ = position.occupied();
    let mut removed = Bitboard(0);

    let mut attacks = pawn_attacks(to, White) & position.pieces[Pawn(White)]
        | pawn_attacks(to, Black) & position.pieces[Pawn(Black)]
        | knight_attacks(to) & knights
        | bishop_attacks(to, occ) & (bishops | queens)
        | rook_attacks(to, occ) & (rooks | queens);

    gain[depth] = target.value();

    'swap: loop {
        depth += 1;
        side = !side;

        gain[depth] = attacker.value() - gain[depth - 1];

        if max(-gain[depth - 1], gain[depth]) < 0 {
            break;
        }

        attacks ^= from_bb;
        occ ^= from_bb;
        removed |= from_bb;

        if may_xray.intersects(from_bb) {
            if let Rook(_) | Queen(_) = attacker {
                attacks |= rook_attacks(to, occ) & (rooks | queens) & !removed;
            }

            if let Pawn(_) | Bishop(_) | Queen(_) = attacker {
                attacks |= bishop_attacks(to, occ) & (bishops | queens) & !removed;
            }
        }

        for piece in Piece::iter_colour(side) {
            let intersection = attacks & position.pieces[piece];
            if !intersection.is_empty() {
                from_bb = Bitboard::from(intersection.get_lsb().unwrap());
                attacker = piece;
                continue 'swap;
            }
        }

        break;
    }

    while {
        depth -= 1;
        depth > 0
    } {
        gain[depth - 1] = -max(-gain[depth - 1], gain[depth]);
    }

    gain[0]
}

fn detect_repetition(pos: &Position, history: [u64; MAX_GAME_PLY], is_root: bool) -> bool {
    if pos.ply - pos.last_irreversible_ply < 4 {
        return false;
    }

    let mut count = 0;
    for ply in (pos.last_irreversible_ply..=pos.ply).rev().step_by(2) {
        if history[ply as usize] == pos.hash {
            count += 1;
        }

        if count >= 2 || (count == 1 && !is_root) {
            return true;
        }
    }

    false
}

pub fn mvv_lva(mv: &Move) -> u8 {
    let attacker = mv.piece;
    let (MoveKind::Capture(victim) | MoveKind::PromotionCapture(_, victim)) = mv.kind else {
        return 0;
    };

    MVV_LVA_TBL[victim][attacker]
}

#[allow(clippy::zero_prefixed_literal)]
const MVV_LVA_TBL: [[u8; 6]; 6] = [
    //p   n   b   r   q   k   attacker
    [06, 05, 04, 03, 02, 01], // pawn victim
    [12, 11, 10, 09, 08, 07], // knight victim
    [18, 17, 16, 15, 14, 13], // bishop victim
    [24, 23, 22, 21, 20, 19], // rook victim
    [30, 29, 28, 27, 26, 25], // queen victim
    [00, 00, 00, 00, 00, 00], // king victim
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{bitboard::Square, position::Position, search::static_exchange_evaluation};

    #[test]
    fn static_exchange_evaluation_test() {
        let mut position = Position::from_fen("1k1r4/1pp4p/p7/4p3/8/P5P1/1PP4P/2K1R3 w - -");
        println!("{position}");
        let from = Square::from_algebraic("e1").unwrap();
        let to = Square::from_algebraic("e5").unwrap();
        let eval = static_exchange_evaluation(&position, from, to, Rook(White), Pawn(Black));
        assert_eq!(eval, 82);

        position.read_fen("1k1r3q/1ppn3p/p4b2/4p3/8/P2N2P1/1PP1R1BP/2K1Q3 w - -");
        println!("{position}");
        let from = Square::from_algebraic("d3").unwrap();
        let eval = static_exchange_evaluation(&position, from, to, Knight(White), Pawn(Black));
        assert_eq!(eval, -255);

        position.read_fen("r1bq1r1k/p1pn1pp1/1p2p3/6b1/3PB3/8/PPPQ1PPP/2KR3R w - - 0 2");
        println!("{position}");
        let from = Square::from_algebraic("d2").unwrap();
        let to = Square::from_algebraic("g5").unwrap();
        let eval = static_exchange_evaluation(&position, from, to, Queen(White), Bishop(Black));
        assert_eq!(eval, -660);
    }
}
