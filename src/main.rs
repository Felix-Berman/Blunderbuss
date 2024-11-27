use perft::{perft, perft_divide};
use position::{Position, STARTING_FEN};

mod bitboard;
mod magic;
mod make_move;
mod move_types;
mod movegen;
mod perft;
mod piece;
mod position;
mod square;

fn main() {
    let mut pos = Position::new();
    _ = pos.read_fen("8/1n4N1/2k5/8/8/5K2/1N4n1/8 w - - 0 1");
    println!("{}\n{}", pos.draw_board(), pos.write_fen());
}
