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
    _ = pos.read_fen("8/8/8/8/8/8/1k6/R3K3 b Q - 0 1");
    println!("{}\n{}", pos.draw_board(), pos.write_fen());
    perft_divide(&mut pos, 2);

    let mv = pos
        .find_algebraic_move("b2a1")
        .expect("should error, not legal");
    pos.make_move(mv);
    println!("{}\n{}", pos.draw_board(), pos.write_fen());
    perft_divide(&mut pos, 1);

    let mv = pos
        .find_algebraic_move("e1c1")
        .expect("should error, not legal");
    pos.make_move(mv);
    println!("{}\n{}", pos.draw_board(), pos.write_fen());
}
