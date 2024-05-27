use std::time::Instant;

use crossbeam_channel::{Receiver, SendError, Sender};

use crate::{engine::MAX_GAME_PLY, eval::evaluate, movegen::{Move, MoveKind, MoveList}, position::Position};

pub const MAX_DEPTH: usize = 64;
const PV_SIZE: usize = MAX_DEPTH * (MAX_DEPTH + 1) / 2 ;
const STALEMATE: i32 = 0;
pub const CHECKMATE: i32 = i32::MAX;
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
    pub history: [u64; MAX_GAME_PLY],
    pub tx: Sender<SendInfo>,
    pub rx: Receiver<SearchCommand>,
    pub stop: bool,
}

impl SearchInfo {
    fn new(stop_nodes: u32,history: [u64; MAX_GAME_PLY], tx: Sender<SendInfo>, rx: Receiver<SearchCommand>) -> Self {
        SearchInfo {
            depth: 0,
            seldepth: 0,
            score: 0,
            nodes: 0,
            stop_nodes,
            time: Instant::now(),
            triangular_pv: [None; PV_SIZE],
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
            time: self.time.elapsed().as_millis() as u32
        };

        self.tx.send(SendInfo::CurrMove(curr_move))
    }

    fn send_bestmove(&self) -> SendResult {
        self.tx.send(SendInfo::Done(self.triangular_pv[0]))
    }

    fn hoist_pv(&mut self, target: usize, source: usize, len: usize) {
        for i in 0..len {
            let Some(mv) = self.triangular_pv[source + i] else {break};
            self.triangular_pv[target + i] = Some(mv);
        }
    }
}

pub fn iterative_deepening(
    mut pos: Position, stop_depth: u8, stop_nodes: u32, history: [u64; MAX_GAME_PLY], 
    tx: Sender<SendInfo>, rx: Receiver<SearchCommand>
) {
    let mut info = SearchInfo::new(stop_nodes, history, tx, rx);

    for depth in 1..=stop_depth {
        info.time = Instant::now();
        info.depth = depth;
        info.nodes = 0;
        info.score = negamax(&mut pos, -i32::MAX, i32::MAX, depth, 0, 0, &mut info);

        info.send_full().unwrap();

        if let Ok(SearchCommand::Stop) = info.rx.try_recv() {
            break
        }

        if info.nodes > info.stop_nodes || info.stop {
            break
        }

        if CHECKMATE - info.score.abs() <= depth as i32 {
            break
        }
    }

    info.send_bestmove().unwrap();
}

fn negamax(pos: &mut Position, mut alpha: i32, beta: i32, depth: u8, ply: usize, pv_idx: usize, info: &mut SearchInfo) -> i32 {
    if ply as u8 > info.seldepth {
        info.seldepth = ply as u8;
    }

    if pos.halfmove >= HALFMOVE_DRAW_COUNT || detect_repetition(pos, info.history) {
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
            continue
        }

        info.history[prev.ply as usize] = prev.hash;
        legal_moves += 1;

        if ply == 0 {
            let _ = info.send_currmove(mv, legal_moves);
        }

        let score = -negamax(pos, -beta, -alpha, depth - 1, ply + 1, next_pv_idx, info);
        info.nodes += 1;

        if score >= beta {
            return beta
        }

        if score > alpha {
            alpha = score;
            info.triangular_pv[pv_idx] = Some(mv);
            info.hoist_pv(pv_idx + 1, next_pv_idx, MAX_DEPTH - ply - 1);
        }

        if info.rx.try_recv().is_ok() || info.nodes > info.stop_nodes || info.stop {
            info.stop = true;
            return alpha
        }

        *pos = prev;
    }

    if legal_moves == 0 {
        if pos.is_check(pos.turn) {
            return -CHECKMATE + ply as i32
        } else {
            return STALEMATE
        }
    }

    alpha
}

fn quiescence_search(pos: &mut Position, mut alpha: i32, beta: i32, ply: usize, info: &mut SearchInfo) -> i32 {
    if ply as u8 > info.seldepth {
        info.seldepth = ply as u8;
    }

    let standing_pat = evaluate(pos);
    if standing_pat >= beta {
        return beta
    }

    if standing_pat > alpha {
        alpha = standing_pat;
    }

    let mut captures = MoveList::new();
    pos.gen_captures(&mut captures);
    captures.score(ply, info);

    for capture in captures {
        let prev = pos.make_move(capture);
        if pos.is_check(prev.turn) {
            *pos = prev;
            continue
        }

        // delta pruning. Need to consider effect on endgame
        if let MoveKind::Capture(piece) | MoveKind::PromotionCapture(_, piece) = capture.kind {
            if standing_pat + piece.value() + 200 <= alpha {
                *pos = prev;
                continue
            }
        }

        info.nodes += 1;
        let score = -quiescence_search(pos, -beta, -alpha, ply + 1, info);
        *pos = prev;

        if score >= beta {
            return beta
        }

        if score > alpha {
            alpha = score;
        }
    }

    alpha
}

fn detect_repetition(pos: &Position, history: [u64; MAX_GAME_PLY]) -> bool {
    if pos.ply - pos.last_irreversible < 4 {
        return false
    }

    for ply in (pos.last_irreversible..pos.ply).step_by(2) {
        if history[ply as usize] == pos.hash {
            return true
        }
    }
    
    false
}


pub fn mvv_lva(mv: &Move) -> u8 {
    let attacker = mv.piece;
    let (MoveKind::Capture(victim) | MoveKind::PromotionCapture(_, victim)) = mv.kind else {
        return 0
    };

    MVV_LVA_TBL[victim][attacker]
}

#[allow(clippy::zero_prefixed_literal)]
const MVV_LVA_TBL: [[u8; 6]; 6] = [
//    p   n   b   r   q   k   attacker
    [06, 05, 04, 03, 02, 01], // pawn victim
    [12, 11, 10, 09, 08, 07], // knight victim
    [18, 17, 16, 15, 14, 13], // bishop victim
    [24, 23, 22, 21, 20, 19], // rook victim
    [30, 29, 28, 27, 26, 25], // queen victim
    [00, 00, 00, 00, 00, 00], // king victim
];
