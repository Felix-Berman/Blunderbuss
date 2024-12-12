use uci::uci_loop;

mod bitboard;
mod evaluate;
mod magic;
mod make_move;
mod move_types;
mod movegen;
mod perft;
mod piece;
mod position;
mod search;
mod square;
mod uci;
mod zobrist;

fn main() {
    match uci_loop() {
        Ok(()) => (),
        Err(err) => println!("info string Error: {:?}", err),
    }
}
