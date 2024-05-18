use std::{error::Error, io, thread::{self, JoinHandle}, time::Instant};

use crossbeam_channel::{unbounded, Sender, Receiver};

use crate::{
    eval::evaluate, 
    fen::STARTING_FEN, 
    interface::{id, parse_command, write_currmove_info, write_full_info, Command::*, SearchControl}, 
    perft::perft_divide, 
    position::{Colour, Position}, 
    search::{iterative_deepening, SearchCommand, SendInfo::{self, *}} 
};

pub const MAX_GAME_PLY: usize = 256; 

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
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        
        let stdin_rx = spawn_reader();
        
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
                            self.history = history;
                        },
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
                    }
                }
            }

            for info in self.info_rx.try_iter() {
                match info {
                    Full(info) => write_full_info(info),
                    CurrMove(info) => if self.search_time.elapsed().as_millis() > 1000 {
                        write_currmove_info(info)
                    },
                    Done(mv) => {
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

            if self.max_time != 0 && self.search_time.elapsed().as_millis() as u32 > self.max_time {
                self.search_tx.send(SearchCommand::Stop)?;
                self.max_time = 0;
            }
        }

        Ok(())
    }


    fn search(&mut self, control: SearchControl) {
        if self.search_handle.is_some() {
            return
        }

        let position = self.position;
        let tx = self.info_tx.clone();
        let rx = self.search_rx.clone();

        self.search_time = Instant::now();
        let history = self.history.clone();
        let handle = thread::spawn(
            move || iterative_deepening(position, control.depth, control.nodes, history, tx, rx)
        );

        self.search_handle = Some(handle);

        self.set_search_limit(control);
    }

    fn set_search_limit(&mut self, control: SearchControl) {
        if control.infinite {
            return
        }

        self.max_time = match self.position.turn {
            Colour::White => calculate_allowed_time(control.wtime, control.winc, control.movestogo),
            Colour::Black => calculate_allowed_time(control.btime, control.binc, control.movestogo),
        };
        
        if control.movetime != 0 {
            self.max_time = control.movetime;
        }
    }

}

fn calculate_allowed_time(time: u32, _inc: u32, mut movestogo: u8) -> u32 {
    if movestogo == 0 {
        movestogo = 40;
    }

    time/movestogo as u32 - 500
}

fn spawn_reader() -> Receiver<String> {
    let (tx, rx) = unbounded::<String>();
    thread::spawn(move || loop {
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();
        tx.send(buffer).unwrap();
    });

    rx
}