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

use engine::Engine;
use zobrist::ZobristCodes;

fn main() {
    ZobristCodes::init();
    let mut engine = Engine::init();
    if let Err(e) = engine.run() {
        println!("{:?}", e);
    }
}
