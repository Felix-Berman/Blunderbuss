use std::time::Instant;

use num::{signum, FromPrimitive};
use lazy_static::lazy_static;
use crate::bitboard::{Bitboard, Square};

lazy_static! {
    pub static ref MAGICS: Magics = Magics::init();
}

const SEED: u64 = 18401105770426537108;
const MAX_ROOK_BITS: usize = 1 << 12;
const MAX_BISHOP_BITS: usize = 1 << 9;

pub struct Magics {
    pub rook_magics: Box<[Magic]>,
    pub bishop_magics: Box<[Magic]>,
    pub rook_attacks: Vec<Vec<Bitboard>>,
    pub bishop_attacks: Vec<Vec<Bitboard>>,
}

impl Magics {
    fn init() -> Self {
        let (rook_attacks, rook_magics) = gen_rook_magics();
        let (bishop_attacks, bishop_magics) = gen_bishop_magics();
        Magics {
            rook_magics: Box::new(rook_magics),
            bishop_magics: Box::new(bishop_magics),
            rook_attacks,
            bishop_attacks
        }
    }
}

#[derive(Clone, Copy)]
pub struct Magic {
    pub mask: Bitboard,
    pub magic: u64,
}

impl Magic {
    fn new() -> Self {
        Magic {mask: Bitboard(0), magic: 0}
    }
}

fn gen_rook_magics() -> (Vec<Vec<Bitboard>>, [Magic; 64]) {
    let mut magics = [Magic::new(); 64];
    let mut attacks = vec![vec![Bitboard(0); MAX_ROOK_BITS]; 64];
    for i in 0..64 {
        let sq = Square::from_usize(i).unwrap();
        (attacks[i], magics[i]) = find_magic_number_rook(sq, SEED);
    }

    (attacks, magics)
}

fn gen_bishop_magics() -> (Vec<Vec<Bitboard>>, [Magic; 64]) {
    let mut magics = [Magic::new(); 64];
    let mut attacks = vec![vec![Bitboard(0); MAX_BISHOP_BITS]; 64];
    for i in 0..64 {
        let sq = Square::from_usize(i).unwrap();
        (attacks[i], magics[i]) = find_magic_number_bishop(sq, SEED);
    }

    (attacks, magics)
}

struct XorShift {
    pub state: u64
}

impl XorShift {
    fn new() -> Self {
        XorShift { state: SEED }
    }

    fn gen_next(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    fn gen_magic(&mut self) -> u64 {
        let mut magic = self.gen_next();
        // magic numbers with few 1 bits are more likely to be successful
        for _ in 0..2 {
            magic &= self.gen_next();
        }

        magic
    }
}

pub fn bishop_attacks(from_sq: Square, blockers: Bitboard) -> Bitboard {
    let mut attacks = Bitboard(0);

    let directions = [9, 7, -9, -7];
    for direction in directions {
        let mut to_sq = from_sq.add(direction);
        let mut prev_rank = from_sq.rank();
        while let Some(sq) = to_sq {
            // break if rank hasn't changed by 1 to handle edge wraps
            if sq.rank() - prev_rank != signum(direction) {
                break;
            }
            attacks.set(sq);
            to_sq = sq.add(direction);
            prev_rank = sq.rank();

            if blockers.is_set(sq) {
                break;
            }
        }
    }

    attacks
}

pub fn rook_attacks(from_sq: Square, blockers: Bitboard) -> Bitboard {
    let mut attacks = Bitboard(0);

    let directions = [1, -1, 8, -8];
    for direction in directions {
        let mut to_sq = from_sq.add(direction);
        while let Some(sq) = to_sq {
            // break if not same rank and file to handle edge wraps
            if from_sq.rank() != sq.rank() && from_sq.file() != sq.file() {
                break;
            }
            attacks.set(sq);
            to_sq = sq.add(direction);
            if blockers.is_set(sq) {
                break;
            }
        }
    }

    attacks
}

pub fn bishop_attacks_mask(from_sq: Square) -> Bitboard {
    let mut attacks = Bitboard(0);

    let directions = [9, 7, -9, -7];
    for direction in directions {
        let mut to_sq = from_sq.add(direction);
        let mut prev_rank = from_sq.rank();
        while let Some(sq) = to_sq {
            // break if rank hasn't changed by 1 to handle edge wraps
            if sq.rank() - prev_rank != signum(direction) {
                break;
            }
            if [0, 7].contains(&sq.rank()) || [0, 7].contains(&sq.file()) {
                break;
            }
            attacks.set(sq);
            to_sq = sq.add(direction);
            prev_rank = sq.rank();
        }
    }

    attacks
}

pub fn rook_attacks_mask(from_sq: Square) -> Bitboard {
    let mut attacks = Bitboard(0);

    let directions = [1, -1, 8, -8];
    for direction in directions {
        let mut to_sq = from_sq.add(direction);
        while let Some(sq) = to_sq {
            // break if not same rank and file to handle edge wraps
            if from_sq.rank() != sq.rank() && from_sq.file() != sq.file() {
                break;
            }
            // break if on eadge
            if ([8, -8].contains(&direction) && [0, 7].contains(&sq.rank())) 
            || ([1, -1].contains(&direction) && [0, 7].contains(&sq.file())) {
                break;
            }
            attacks.set(sq);
            to_sq = sq.add(direction);
        }
    }

    attacks
}

fn gen_occupancy(index: usize, mask: Bitboard) -> Bitboard {
    let mut occupancy = Bitboard(0);

    for (n, sq) in mask.enumerate() {
        if index & 1 << n != 0 {
            occupancy.set(sq);
        } 
    }

    occupancy
}

pub fn find_magic_number_rook(sq: Square, seed: u64) -> (Vec<Bitboard>, Magic) {
    let mask = rook_attacks_mask(sq);
    let mut occupancies = [Bitboard(0); MAX_ROOK_BITS];
    let mut attacks_by_occupancy = [Bitboard(0); MAX_ROOK_BITS];

    let num_bit_combinations = 1 << mask.count_ones();
    for i in 0..num_bit_combinations {
        occupancies[i] = gen_occupancy(i, mask);
        attacks_by_occupancy[i] = rook_attacks(sq, occupancies[i]);
    }

    let mut prng = XorShift::new();
    prng.state = seed;
    'magic_search: loop {
        let magic = prng.gen_magic();

        let (mul, _) = mask.0.overflowing_mul(magic);
        if (mul & 0xFF00000000000000).count_ones() < 6 {
            continue 'magic_search
        }

        let mut attacks_by_magic = [Bitboard(0); MAX_ROOK_BITS];

        for i in 0..num_bit_combinations {
            let (mul, _) = occupancies[i].0.overflowing_mul(magic);
            let magic_idx = (mul >> (64 - ROOK_BITS[sq])) as usize;

            if !attacks_by_magic[magic_idx].is_empty() {
                continue 'magic_search // magic number failed to uniquely index attacks by occupancy
            }
            attacks_by_magic[magic_idx] = attacks_by_occupancy[i];
        }

        return (attacks_by_magic.to_vec(), Magic { mask, magic })
    }
}

pub fn find_magic_number_bishop(sq: Square, seed: u64) -> (Vec<Bitboard>, Magic) {
    let mask = bishop_attacks_mask(sq);
    let mut occupancies = [Bitboard(0); MAX_BISHOP_BITS];
    let mut attacks_by_occupancy = [Bitboard(0); MAX_BISHOP_BITS];

    let num_bit_combinations = 1 << mask.count_ones();
    for i in 0..num_bit_combinations {
        occupancies[i] = gen_occupancy(i, mask);
        attacks_by_occupancy[i] = bishop_attacks(sq, occupancies[i]);
    }

    let mut prng = XorShift::new();
    prng.state = seed;
    'magic_search: loop {
        let magic = prng.gen_magic();

        let (mul, _) = mask.0.overflowing_mul(magic);
        if (mul & 0xFF00000000000000).count_ones() < 6 {
            continue 'magic_search
        }

        let mut attacks_by_magic = [Bitboard(0); MAX_BISHOP_BITS];

        for i in 0..num_bit_combinations {
            let (mul, _) = occupancies[i].0.overflowing_mul(magic);
            let magic_idx = (mul >> (64 - BISHOP_BITS[sq])) as usize;

            if !attacks_by_magic[magic_idx].is_empty() {
                continue 'magic_search
            }
            attacks_by_magic[magic_idx] = attacks_by_occupancy[i];
        }

        return (attacks_by_magic.to_vec(), Magic { mask, magic })
    }
}


pub fn find_best_seed() -> u64 {
    let mut prng = XorShift::new();
    let mut best_time = u128::MAX;
    let mut best_seed = 0;

    for _ in 0..20 {
        let seed = prng.gen_next();
        let timer = Instant::now();
        for sq in 0..64 {
            let (_, rook) = find_magic_number_rook(Square::from_u8(sq).unwrap(), seed);
            let (_, bishop) = find_magic_number_bishop(Square::from_u8(sq).unwrap(), seed);
            println!("sq {} rook {} bishop {}", sq, rook.magic, bishop.magic);
        }

        let elapsed = timer.elapsed().as_millis();
        if elapsed < best_time {
            best_time = elapsed;
            best_seed = seed;
        }
    } 

    best_seed
}

pub const ROOK_BITS: [u8; 64] = [
  12, 11, 11, 11, 11, 11, 11, 12,
  11, 10, 10, 10, 10, 10, 10, 11,
  11, 10, 10, 10, 10, 10, 10, 11,
  11, 10, 10, 10, 10, 10, 10, 11,
  11, 10, 10, 10, 10, 10, 10, 11,
  11, 10, 10, 10, 10, 10, 10, 11,
  11, 10, 10, 10, 10, 10, 10, 11,
  12, 11, 11, 11, 11, 11, 11, 12
];

pub const BISHOP_BITS: [u8; 64] = [
  6, 5, 5, 5, 5, 5, 5, 6,
  5, 5, 5, 5, 5, 5, 5, 5,
  5, 5, 7, 7, 7, 7, 5, 5,
  5, 5, 7, 9, 9, 7, 5, 5,
  5, 5, 7, 9, 9, 7, 5, 5,
  5, 5, 7, 7, 7, 7, 5, 5,
  5, 5, 5, 5, 5, 5, 5, 5,
  6, 5, 5, 5, 5, 5, 5, 6
];

#[cfg(test)]
mod tests {
    use num::FromPrimitive;

    use super::*;

    #[test]
    fn rook_mask_generation() {
        for sq in 0..64 {
            println!("testing sq {}", sq);
            let mask = rook_attacks_mask(Square::from_u8(sq).unwrap());
            println!("{}", mask);
        }
    }

    #[test]
    fn bishop_mask_generation() {
        for sq in 0..64 {
            println!("testing sq {}", sq);
            let mask = bishop_attacks_mask(Square::from_u8(sq).unwrap());
            println!("{}", mask);
        }
    }

    #[test]
    fn magic_bishop_generation() {
        for sq in 0..64 {
            println!("testing sq {}", sq);
            let (_, magic_bishop) = find_magic_number_bishop(Square::from_u8(sq).unwrap(), SEED);
            println!("found magic bishop {}", magic_bishop.magic);
        }
    }

    #[test]
    fn magic_rook_generation() {
        for sq in 0..64 {
            println!("testing sq {}", sq);
            let (_, magic_rook) = find_magic_number_rook(Square::from_u8(sq).unwrap(), SEED);
            println!("found magic rook {}", magic_rook.magic);
        }
    }
}
