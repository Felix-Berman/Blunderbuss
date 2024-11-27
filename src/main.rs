use evaluate::evaluate;
use position::{Position, STARTING_FEN};

mod bitboard;
mod evaluate;
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
    _ = pos.read_fen(STARTING_FEN);

    println!("{}\n{}", pos.draw_board(), pos.write_fen());
    println!("evaluation: {}", evaluate(&pos))
}
