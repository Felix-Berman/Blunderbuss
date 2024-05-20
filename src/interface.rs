// uci protocol https://www.wbec-ridderkerk.nl/html/UCIProtocol.html

use std::str::SplitWhitespace;

use itertools::Itertools;

use crate::{
    engine::MAX_GAME_PLY, 
    fen::STARTING_FEN,  
    position::Position, 
    search::{CurrMoveInfo, FullInfo, CHECKMATE, MAX_DEPTH}
};

#[derive(Debug)]
pub enum Command {
    Uci,
    Debug(bool),
    IsReady,
    _SetOption(EngineOption),
    UCINewGame,
    Position(Position, Box<[u64; MAX_GAME_PLY]>),
    Go(SearchControl),
    Stop,
    PonderHit,
    Quit,
    Print,
    Perft(u8),
    Evaluate,
    Move(String),
}

#[derive(Default, Debug)]
pub struct SearchControl {
    pub nodes: u32,
    pub depth: u8,
    pub movetime: u32,
    pub wtime: u32,
    pub btime: u32,
    pub winc: u32,
    pub binc: u32,
    pub movestogo: u8,
    pub mate: u8,
    pub infinite: bool,
    pub ponder: bool,
}

impl SearchControl {
    pub fn new() -> Self {
        Self {
            nodes: u32::MAX,
            depth: MAX_DEPTH as u8,
            movetime: 0,
            wtime: 0,
            btime: 0,
            winc: 0,
            binc: 0,
            movestogo: 0,
            mate: 0,
            infinite: false,
            ponder: false,
        }
    }
}

#[derive(Debug)]
pub enum EngineOption {}

pub fn parse_command(line: &str) -> Option<Command> {
    let mut tokens = line.split_whitespace();

    let cmd = match tokens.next()? {
        "uci" => Command::Uci,
        "debug" => Command::Debug(tokens.next()? == "on"),
        "isready" => Command::IsReady,
        "setoption" => todo!(),
        "ucinewgame" => Command::UCINewGame,
        "position" => position(tokens)?,
        "go" => go(tokens)?,
        "stop" => Command::Stop,
        "ponderhit" => Command::PonderHit,
        "quit" => Command::Quit,
        "print" => Command::Print,
        "perft" => Command::Perft(tokens.next()?.parse().ok()?),
        "eval" => Command::Evaluate,
        "move" => Command::Move(tokens.next()?.to_string()),
        _ => return None
    };

    Some(cmd)
}

fn position(mut tokens: SplitWhitespace) -> Option<Command> {
    let mut position = match tokens.next()? {
        "startpos" => Position::from_fen(STARTING_FEN),
        "fen" => Position::from_fen(&tokens.clone().take_while(|s| *s != "moves").join(" ")),
        _ => return None
    };

    let mut history = [0; MAX_GAME_PLY];
    while let Some(str) = tokens.next() {
        if str != "moves" {
            continue
        }

        for mv_str in tokens.by_ref() {
            let mv = position.find_algebraic_move(mv_str)?;
            history[position.ply as usize] = position.hash;
            position.make_move(mv);
        }
    }

    Some(Command::Position(position, Box::new(history)))
}

fn go(mut tokens: SplitWhitespace) -> Option<Command> {
    let mut next = tokens.next();

    let mut control = SearchControl::new();

    while let Some(token) = next {
        match token {
            "nodes" => control.nodes = tokens.next()?.parse().ok()?,
            "depth" => control.depth = tokens.next()?.parse().ok()?,
            "movetime" => control.movetime = tokens.next()?.parse().ok()?,
            "wtime" => control.wtime = tokens.next()?.parse().ok()?,
            "btime" => control.btime = tokens.next()?.parse().ok()?,
            "winc" => control.winc = tokens.next()?.parse().ok()?,
            "binc" => control.binc = tokens.next()?.parse().ok()?,
            "movestogo" => control.movestogo = tokens.next()?.parse().ok()?,
            "mate" => control.mate = tokens.next()?.parse().ok()?,
            "infinite" => control.infinite = true,
            "ponder" => control.ponder = true,
            _ => return None
        }

        next = tokens.next();
    }

    Some(Command::Go(control))
}

pub fn id() {
    println!("id name Blunderbuss");
    println!("id author Felix Berman");
    println!("uciok");
}

pub fn write_full_info(info: FullInfo) {
    let time = if info.time > 0 {
        info.time
    } else {
        1
    };
    let nps = info.nodes * 1000 / time;

    let distance_from_mate = CHECKMATE - info.score.abs();
    let score = if distance_from_mate <= info.depth as i32 {
        format!("mate {}", distance_from_mate * info.score.signum())
    } else {
        format!("cp {}", info.score)
    };

    let pv = info.pv.iter().filter_map(|mv| *mv);

    println!(
        "info depth {} seldepth {} score {} nodes {} nps {} time {} pv {}",
        info.depth, info.seldepth, score, info.nodes, nps, info.time, pv.format(" ")
    );
}

pub fn write_currmove_info(info: CurrMoveInfo) {
    println!(
        "info depth {} currmove {} currmovenumber {}",
        info.depth, info.mv, info.mv_num
    );
}