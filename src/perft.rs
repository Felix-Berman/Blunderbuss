use crate::position::Position;

pub fn perft_divide(pos: &mut Position, depth: u8) {
    let mut total_nodes = 0;
    let mut moves = Vec::new();

    pos.gen_moves(&mut moves);

    for mv in moves {
        let prev = pos.make_move(mv);
        if pos.is_check(!pos.turn) {
            *pos = prev;
            continue
        }
        let nodes = perft(pos, depth - 1);
        println!("{} {}", mv, nodes);
        total_nodes += nodes;
        *pos = prev;
    }

    println!("\n{}", total_nodes);
}

pub fn perft(pos: &mut Position, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut nodes = 0;

    let mut moves = Vec::new();
    pos.gen_moves(&mut moves);
    for mv in moves {
        let prev = pos.make_move(mv);
        if pos.is_check(!pos.turn) {
            *pos = prev;
            continue
        }
        nodes += perft(pos, depth - 1);
        *pos = prev;
    }

    nodes
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
        position.read_fen(fen);

        for depth in 1..test.len() {
            let nodes = perft(&mut position, depth as u8);
            let expected = test[depth].split_whitespace().collect::<Vec<&str>>()[1]
                .parse()
                .unwrap();
            if nodes != expected {
                println!("{}", position);
                println!("depth: {}", depth);
                panic!("found {} nodes\nexpected {} nodes", nodes, expected);
            }
        }

        Ok(())
    }
    test_cases!(0,50);
}