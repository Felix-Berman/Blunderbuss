use position::{Position, STARTING_FEN};

mod bitboard;
mod magic;
mod make_move;
mod move_types;
mod movegen;
mod piece;
mod position;
mod square;

fn main() {
    let mut pos = Position::new();
    _ = pos.read_fen(STARTING_FEN);
    let board_str = pos.draw_board();
    println!("{board_str}");

    let moves = pos.gen_moves();
    for mv in moves {
        println!("{mv}");
    }
    println!("moves: {}", moves.length);

    pos.make_move(moves[0]);
    println!("{}", pos.draw_board());
}
