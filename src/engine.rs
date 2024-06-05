use std::{
    error::Error,
    io,
    thread::{self, JoinHandle},
    time::Instant,
};

use crossbeam_channel::{unbounded, Receiver, Sender};

use crate::{
    eval::evaluate,
    fen::STARTING_FEN,
    interface::{
        id, parse_command, write_currmove_info, write_full_info, Command::*, SearchControl,
    },
    perft::perft_divide,
    position::{Colour, Position},
    search::{iterative_deepening, CurrMoveInfo, SearchCommand, SendInfo},
};

pub const MAX_GAME_PLY: usize = 256;
pub const CURRMOVE_WAIT_TIME: u32 = 3000;

pub struct Engine {
    pub debug: bool,
    pub position: Position,
    pub search_handle: Option<JoinHandle<()>>,
    pub max_time: u32,
    pub search_time: Instant,
    pub search_tx: Sender<SearchCommand>,
    pub search_rx: Receiver<SearchCommand>,
    pub info_tx: Sender<SendInfo>,
    pub info_rx: Receiver<SendInfo>,
    pub history: [u64; MAX_GAME_PLY],
    pub nodes: u32,
    pub currmove_buffer: Vec<CurrMoveInfo>,
}

impl Engine {
    pub fn init() -> Self {
        let (search_tx, search_rx) = unbounded::<SearchCommand>();
        let (info_tx, info_rx) = unbounded::<SendInfo>();

        Self {
            debug: false,
            position: Position::from_fen(STARTING_FEN),
            search_handle: None,
            max_time: 0,
            search_time: Instant::now(),
            search_tx,
            search_rx,
            info_tx,
            info_rx,
            history: [0; MAX_GAME_PLY],
            nodes: 0,
            currmove_buffer: Vec::new(),
        }
    }

    pub fn run(&mut self, cmd_args: String) -> Result<(), Box<dyn Error>> {
        let (stdin_rx, stdin_tx) = spawn_reader();
        stdin_tx.send(cmd_args)?;

        'running: loop {
            if let Ok(input) = stdin_rx.try_recv() {
                if let Some(cmd) = parse_command(&input) {
                    match cmd {
                        Uci => id(),
                        Debug(d) => self.debug = d,
                        IsReady => println!("readyok"),
                        _SetOption(_) => todo!("no options configured yet"),
                        UCINewGame => self.position = Position::new(),
                        Position(position, history) => {
                            self.position = position;
                            self.history = *history;
                        }
                        Go(control) => self.search(control),
                        Stop => self.search_tx.send(SearchCommand::Stop)?,
                        PonderHit => todo!("no pondering configured yet"),
                        Quit => break 'running,
                        Print => println!("{}", self.position),
                        Perft(depth) => perft_divide(&mut self.position, depth),
                        Evaluate => println!("{}", evaluate(&self.position)),
                        Move(mv_str) => {
                            if let Some(mv) = self.position.find_algebraic_move(&mv_str) {
                                self.position.make_move(mv);
                            }
                        }
                        Benchmark => self.benchmark(),
                    }
                }
            }

            self.receive_info();

            if self.max_time != 0 && self.search_time.elapsed().as_millis() as u32 > self.max_time {
                self.search_tx.send(SearchCommand::Stop)?;
                self.max_time = 0;
            }
        }

        Ok(())
    }

    pub fn search(&mut self, control: SearchControl) {
        if self.search_handle.is_some() {
            return;
        }

        let position = self.position;
        let tx = self.info_tx.clone();
        let rx = self.search_rx.clone();

        self.search_time = Instant::now();
        let history = self.history;
        let handle = thread::spawn(move || {
            iterative_deepening(position, control.depth, control.nodes, history, tx, rx)
        });

        self.search_handle = Some(handle);

        self.set_search_limit(control);
    }

    pub fn set_search_limit(&mut self, control: SearchControl) {
        if control.infinite {
            return;
        }

        self.max_time = match self.position.turn {
            Colour::White => calculate_allowed_time(control.wtime, control.winc, control.movestogo),
            Colour::Black => calculate_allowed_time(control.btime, control.binc, control.movestogo),
        };

        if control.movetime != 0 {
            self.max_time = control.movetime;
        }
    }

    pub fn receive_info(&mut self) {
        for info in self.info_rx.try_iter() {
            match info {
                SendInfo::Full(info) => {
                    self.nodes += info.nodes;
                    write_full_info(info);
                    self.currmove_buffer.drain(..);
                }
                SendInfo::CurrMove(info) => {
                    if self.search_time.elapsed().as_millis() > CURRMOVE_WAIT_TIME.into() {
                        for info in self.currmove_buffer.drain(..) {
                            write_currmove_info(info);
                        }
                        write_currmove_info(info)
                    } else {
                        self.currmove_buffer.push(info);
                    }
                }
                SendInfo::Done(mv) => {
                    if let Some(mv) = mv {
                        println!("bestmove {}", mv);
                    } else {
                        println!("bestmove None");
                    }
                    let handle = self.search_handle.take().unwrap();
                    handle.join().unwrap();
                }
            }
        }
    }
}

fn calculate_allowed_time(time: u32, _inc: u32, mut movestogo: u8) -> u32 {
    if movestogo == 0 {
        movestogo = 40;
    }

    time / (movestogo + 2) as u32
}

fn spawn_reader() -> (Receiver<String>, Sender<String>) {
    let (tx, rx) = unbounded::<String>();
    let tx_clone = tx.clone();
    thread::spawn(move || loop {
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();
        tx_clone.send(buffer).unwrap();
    });

    (rx, tx)
}
