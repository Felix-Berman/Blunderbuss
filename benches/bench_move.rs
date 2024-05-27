use blunderbuss::position::Position;

fn main() {
    divan::main();
}

#[divan::bench]
fn move_bench() {
    let position = Position::from_fen("r1b2rk1/2q1b1pp/p2ppn2/1p6/3QP3/1BN1B3/PPP3PP/R4RK1 w - - 0 1");
    _ = position.gen_moves(); 
}
