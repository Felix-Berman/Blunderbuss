use position::Position;

mod bitboard;
mod piece;
mod position;
mod square;

fn main() {
    let mut pos = Position::new();
    pos.read_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
}
