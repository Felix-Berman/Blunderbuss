use position::Position;

mod bitboard;
mod magic;
mod move_types;
mod movegen;
mod piece;
mod position;
mod square;

fn main() {
    let mut pos = Position::new();
    _ = pos.read_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let board_str = pos.draw_board().unwrap();
    println!("{board_str}")
}
