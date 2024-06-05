use blunderbuss::engine::Engine;
use blunderbuss::zobrist::ZobristCodes;

fn main() {
    let args: String = std::env::args().collect();

    ZobristCodes::init();
    let mut engine = Engine::init();
    if let Err(e) = engine.run(args) {
        println!("{:?}", e);
    }
}
