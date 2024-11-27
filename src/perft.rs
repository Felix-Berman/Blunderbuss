use crate::position::Position;

pub fn perft(pos: &mut Position, depth: u8) -> u32 {
    if depth == 0 {
        return 1;
    }

    let mut nodes = 0;

    let moves = pos.gen_moves();
    for mv in moves {
        let mut next_pos = *pos;
        next_pos.make_move(mv);
        if next_pos.is_check(!next_pos.active_colour) {
            continue;
        }

        nodes += perft(&mut next_pos, depth - 1);
    }

    nodes
}

pub fn perft_divide(pos: &mut Position, depth: u8) {
    let mut total_nodes = 0;

    let moves = pos.gen_moves();
    for mv in moves {
        let mut next_pos = *pos;
        next_pos.make_move(mv);
        if next_pos.is_check(!next_pos.active_colour) {
            continue;
        }

        let nodes = perft(pos, depth - 1);
        println!("{} {}", mv, nodes);
        total_nodes += nodes;
    }

    println!("\n{}", total_nodes);
}
