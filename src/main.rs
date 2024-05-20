mod bitboard;
mod movegen;
mod position;
mod fen;
mod make_move;
mod perft;
mod interface;
mod eval;
mod search;
mod engine;
mod zobrist;
mod benchmark;

use engine::Engine;
use zobrist::ZobristCodes;

fn main() {
    let args: String = std::env::args().collect();

    ZobristCodes::init();
    let mut engine = Engine::init();
    if let Err(e) = engine.run(args) {
        println!("{:?}", e);
    }
}
