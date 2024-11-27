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
    _ = pos.read_fen("R7/8/8/8/8/8/1k6/4K3 b - - 1 1");

    let depth = 2;
    perft_divide(&mut pos, depth);

    println!("{}", perft(&mut pos, depth));
    println!("{}\n{}", pos.draw_board(), pos.write_fen());
    let moves = pos.gen_moves();
    for mv in moves {
        println!("{mv}");
    }
    pos.make_move(moves[0]);

    println!("{}\n{}", pos.draw_board(), pos.write_fen());
    perft_divide(&mut pos, depth - 1);
    println!("{}", perft(&mut pos, depth - 1));
}
