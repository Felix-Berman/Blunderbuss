// implementing the UCI protocol https://www.wbec-ridderkerk.nl/html/UCIProtocol.html

use itertools::Itertools;
use lazy_static::lazy_static;
use std::{
    io,
    str::SplitWhitespace,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

use crate::{
    piece::Colour::*,
    position::{Position, STARTING_FEN},
    search::{root_search, Nodes, Ply, SearchInfo},
};

fn spawn_stdin_channel() -> Receiver<String> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        tx.send(input).unwrap();
    });
    rx
}

pub fn uci_loop() -> io::Result<()> {
    let mut pos = Position::new();
    _ = pos.read_fen(STARTING_FEN);
    let mut stop = Arc::new(AtomicBool::new(false));
    let mut timer = Timer::new();
    let input_channel = spawn_stdin_channel();

    'running: loop {
        if let Ok(input) = input_channel.try_recv() {
            let mut tokens = input.split_whitespace();

            match tokens.next() {
                Some("uci") => id(),
                Some("debug") => todo!(),
                Some("isready") => println!("readyok"),
                Some("setoption") => todo!(),
                Some("ucinewgame") => _ = pos.read_fen(STARTING_FEN),
                Some("position") => position(tokens, &mut pos),
                Some("go") => (stop, timer) = go(tokens, pos),
                Some("stop") => stop.store(true, Ordering::Relaxed),
                Some("ponderhit") => todo!(),
                Some("draw") => println!("{}\n{}", pos.draw_board(), pos.write_fen()),
                Some("quit") => break 'running,
                _ => (),
            }
        }

        if timer.active && timer.is_up() {
            stop.store(true, Ordering::Relaxed);
        }
    }

    Ok(())
}

fn id() {
    println!("id name {}", env!("CARGO_PKG_NAME"));
    println!("id author {}", env!("CARGO_PKG_AUTHORS"));
    println!("uciok");
}

fn position(mut tokens: SplitWhitespace, pos: &mut Position) {
    _ = match tokens.next() {
        Some("startpos") => pos.read_fen(STARTING_FEN),
        Some("fen") => pos.read_fen(&tokens.clone().take_while(|s| *s != "moves").join(" ")),
        _ => return,
    };

    for string in tokens.skip_while(|s| *s != "moves").skip(1) {
        match pos.find_algebraic_move(string) {
            Some(mv) => pos.make_move(mv),
            None => println!("invalid move {}", string),
        }
    }
}

fn go(mut tokens: SplitWhitespace, pos: Position) -> (Arc<AtomicBool>, Timer) {
    let stop = Arc::new(AtomicBool::new(false));
    let mut info = SearchInfo::new(stop.clone());
    let mut white_time = Duration::ZERO;
    let mut black_time = Duration::ZERO;
    let mut white_increment = Duration::ZERO;
    let mut black_increment = Duration::ZERO;
    let mut moves_to_go = 50;
    let mut timer = Timer::new();

    while let Some(token) = tokens.next() {
        match token {
            "searchmoves" => todo!(),
            "ponder" => todo!(),
            "wtime" => white_time = parse_duration(&mut tokens),
            "btime" => black_time = parse_duration(&mut tokens),
            "winc" => white_increment = parse_duration(&mut tokens),
            "binc" => black_increment = parse_duration(&mut tokens),
            "movestogo" => moves_to_go = tokens.next().and_then(|s| s.parse::<u8>().ok()).unwrap(),
            "movetime" => timer.set(parse_duration(&mut tokens)),
            "depth" => info.max_depth = tokens.next().and_then(|s| s.parse::<Ply>().ok()).unwrap(),
            "nodes" => {
                info.max_nodes = tokens.next().and_then(|s| s.parse::<Nodes>().ok()).unwrap()
            }
            "mate" => todo!(),
            "infinite" => (),
            _ => (),
        }
    }

    thread::spawn(move || {
        let mv = root_search(pos, info);
        println!("bestmove {}", mv);
    });

    if !(timer.active || white_time.is_zero() && black_time.is_zero()) {
        let (time, increment) = match pos.active_colour {
            White => (white_time, white_increment),
            Black => (black_time, black_increment),
        };

        let total_time = time + increment * moves_to_go as u32;
        let current_move = pos.ply / 2 + 1;
        let end_ply = current_move + moves_to_go;
        let allowed_time = total_time.mul_f32(percent_time_for_move(pos.ply, end_ply));
        timer.set(allowed_time)
    }

    (stop, timer)
}

fn parse_duration(tokens: &mut SplitWhitespace) -> Duration {
    tokens
        .next()
        .map(|s| Duration::from_millis(s.parse().unwrap()))
        .unwrap()
}

struct Timer {
    start: Instant,
    duration: Duration,
    pub active: bool,
}

impl Timer {
    fn new() -> Timer {
        Timer {
            start: Instant::now(),
            duration: Duration::ZERO,
            active: false,
        }
    }

    fn set(&mut self, time: Duration) {
        self.active = true;
        self.duration = time;
    }

    fn is_up(&self) -> bool {
        self.start.elapsed() >= self.duration
    }
}

// The time allowed as a percentage of the time remaining
// Calculated as the ratio of the weight of current move to the sum of weights until next time
// control
//
fn percent_time_for_move(current_move: Ply, end_move: Ply) -> f32 {
    move_weight[current_move as usize]
        / (current_move..=end_move).fold(0.0, |acc, x| acc + move_weight[x as usize])
}

const SCALE: f32 = 30.0;
const SHIFT: f32 = -5.0;
const PLY_SIZE: usize = Ply::MAX as usize + 1;

lazy_static! {
    static ref move_weight: [f32; PLY_SIZE] =
        core::array::from_fn(|x| time_function((x as f32 - SHIFT) / SCALE));
}

// f(x) = 1/x^{(x-1)}
fn time_function(x: f32) -> f32 {
    1.0 / x.powf(x - 1.0)
}
