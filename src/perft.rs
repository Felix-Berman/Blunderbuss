use std::io;

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

        let nodes = perft(&mut next_pos, depth - 1);
        println!("{} {}", mv, nodes);
        total_nodes += nodes;
    }

    println!("\n{}", total_nodes);
}

#[cfg(test)]
mod tests {
    use super::*;
    use seq_macro::seq;
    use std::env;
    use std::fs;
    use test_case::test_case;

    macro_rules! test_cases {
        ($first:expr, $last:expr) => {
            seq!(N in $first..$last {
                #(#[test_case(N)])*
                fn perft_test_n(n: usize) -> Result<(), Box<dyn std::error::Error>> {
                    perft_test(n)
                }
            });
        };
    }

    fn perft_test(n: usize) -> Result<(), Box<dyn std::error::Error>> {
        let mut path = env::current_dir().unwrap();
        path.push("perftsuite.epd");
        let contents = fs::read_to_string(path)?;
        let tests: Vec<&str> = contents.split("\n").collect();
        let test: Vec<&str> = tests[n].split(";").collect();
        let fen = test[0];

        let mut position = Position::new();
        position.read_fen(fen)?;

        for depth in 1..test.len() {
            let nodes = perft(&mut position, depth as u8);
            let expected = test[depth].split_whitespace().collect::<Vec<&str>>()[1]
                .parse()
                .unwrap();
            assert_eq!(
                nodes,
                expected,
                "depth: {}\nfen: {}",
                depth,
                position.write_fen()
            );
        }

        Ok(())
    }
    test_cases!(0, 126);
}
