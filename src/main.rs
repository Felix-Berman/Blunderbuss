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
    _ = pos.read_fen(STARTING_FEN);
    let mv = pos.find_algebraic_move("a2a3").unwrap();
    pos.make_move(mv);
    let mv = pos.find_algebraic_move("a7a5").unwrap();
    pos.make_move(mv);
    let mv = pos.find_algebraic_move("e2e4").unwrap();
    pos.make_move(mv);
    let mv = pos.find_algebraic_move("f7f5").unwrap();
    pos.make_move(mv);
    let mv = pos.find_algebraic_move("d1h5").unwrap();
    pos.make_move(mv);

    println!("{}\n{}", pos.draw_board(), pos.write_fen());
    perft_divide(&mut pos, 1);
}
