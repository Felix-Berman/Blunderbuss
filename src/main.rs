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
    _ = pos.read_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    let mv = pos.find_algebraic_move("a2a4").unwrap();
    pos.make_move(mv);

    println!("{}\n{}", pos.draw_board(), pos.write_fen());
    perft_divide(&mut pos, 1);
}
