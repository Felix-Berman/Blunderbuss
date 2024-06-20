use std::{env, fs, time::Instant};

use crate::{engine::Engine, interface::SearchControl, position::Position, search::SearchCommand};

const NUM_TESTS: usize = 50;
const TEST_TIME: u32 = 1000;
const TEST_DEPTH: u8 = 6;

impl Engine {
    pub fn benchmark(&mut self) {
        let mut path = env::current_dir().unwrap();
        path.push("arasan2023.epd");
        let contents = fs::read_to_string(path).unwrap();
        let tests: Vec<&str> = contents.split('\n').take(NUM_TESTS).collect();

        let start_time = Instant::now();
        self.nodes = 0;
        for (i, test) in tests.iter().enumerate() {
            let mut test: Vec<String> = test.split(';').map(|s| s.to_string()).collect();
            let bm_offset = test[0].find("bm").unwrap_or(test[0].len());
            let fen: String = test[0].drain(..bm_offset).collect();

            println!("\nTest: {}/{} \"{}\"", i + 1, NUM_TESTS, fen);

            self.position = Position::from_fen(&fen);
            let mut control = SearchControl::new();
            // control.movetime = TEST_TIME;
            control.depth = TEST_DEPTH;

            self.search(control);

            while self.search_handle.is_some() {
                self.receive_info();

                if self.max_time != 0
                    && self.search_time.elapsed().as_millis() as u32 > self.max_time
                {
                    self.search_tx.send(SearchCommand::Stop).unwrap();
                    self.max_time = 0;
                }
            }
        }

        let total_time = start_time.elapsed().as_millis();
        let nps = self.nodes / total_time as u32 * 1000;
        println!("=============================================");
        println!("{} ms, {} nodes, {} nps", total_time, self.nodes, nps);
    }
}
