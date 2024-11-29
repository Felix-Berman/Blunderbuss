// implementing the UCI protocol https://www.wbec-ridderkerk.nl/html/UCIProtocol.html

use itertools::Itertools;
use std::{io, str::SplitWhitespace, thread};

use crate::{
    position::{Position, STARTING_FEN},
    search::{root_search, MAX_DEPTH},
};

pub fn uci_loop() -> io::Result<()> {
    let mut pos = Position::new();

    'running: loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let mut tokens = input.split_whitespace();

        match tokens.next() {
            Some("uci") => id(),
            Some("debug") => todo!(),
            Some("isready") => println!("readyok"),
            Some("setoption") => todo!(),
            Some("ucinewgame") => todo!(),
            Some("position") => position(tokens, &mut pos),
            Some("go") => go(tokens, pos.clone()),
            Some("stop") => todo!(),
            Some("ponderhit") => todo!(),
            Some("draw") => println!("{}\n{}", pos.draw_board(), pos.write_fen()),
            Some("quit") => break 'running,
            _ => (),
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

    for string in tokens.skip_while(|s| *s != "moves") {
        match pos.find_algebraic_move(string) {
            Some(mv) => pos.make_move(mv),
            None => break,
        }
    }
}

fn go(mut tokens: SplitWhitespace, pos: Position) {
    let mut depth = MAX_DEPTH;

    while let Some(token) = tokens.next() {
        match token {
            "searchmoves" => todo!(),
            "ponder" => todo!(),
            "wtime" => todo!(),
            "btime" => todo!(),
            "winc" => todo!(),
            "binc" => todo!(),
            "movestogo" => todo!(),
            "depth" => {
                if let Some(d) = tokens.next().and_then(|s| s.parse::<u8>().ok()) {
                    depth = d;
                }
            }
            "nodes" => todo!(),
            "mate" => todo!(),
            "movetime" => todo!(),
            "infinite" => todo!(),
            _ => (),
        }
    }

    thread::spawn(move || {
        let (mv, score) = root_search(&pos, depth);
        println!("info score cp {}", score);
        println!("bestmove {}", mv);
    });
}
