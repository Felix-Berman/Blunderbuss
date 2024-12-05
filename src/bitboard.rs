use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Mul, Not, Shl,
    ShlAssign, Shr, ShrAssign, Sub, SubAssign,
};

use crate::square::Square;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct Bitboard(u64);

#[allow(dead_code)]
impl Bitboard {
    pub const EMPTY: Bitboard = Bitboard(0);
    pub const A_FILE: Bitboard = Bitboard(0x101010101010101);
    pub const B_FILE: Bitboard = Bitboard(0x202020202020202);
    pub const C_FILE: Bitboard = Bitboard(0x404040404040404);
    pub const D_FILE: Bitboard = Bitboard(0x808080808080808);
    pub const E_FILE: Bitboard = Bitboard(0x1010101010101010);
    pub const F_FILE: Bitboard = Bitboard(0x2020202020202020);
    pub const G_FILE: Bitboard = Bitboard(0x4040404040404040);
    pub const H_FILE: Bitboard = Bitboard(0x8080808080808080);
    pub const RANK_8: Bitboard = Bitboard(0xff);
    pub const RANK_7: Bitboard = Bitboard(0xff << 8);
    pub const RANK_6: Bitboard = Bitboard(0xff << 16);
    pub const RANK_5: Bitboard = Bitboard(0xff << 24);
    pub const RANK_4: Bitboard = Bitboard(0xff << 32);
    pub const RANK_3: Bitboard = Bitboard(0xff << 40);
    pub const RANK_2: Bitboard = Bitboard(0xff << 48);
    pub const RANK_1: Bitboard = Bitboard(0xff << 56);
    pub const BACK_RANKS: Bitboard = Bitboard(0xff | 0xff << 56);
    pub const KINGSIDE_CASTLING: Bitboard = Bitboard(0x60);
    pub const QUEENSIDE_CASTLING: Bitboard = Bitboard(0xe);
    pub const ROOKS: Bitboard = Bitboard(0x8100000000000081);

    pub fn bitscan(self) -> Option<Square> {
        Square::try_from(self.0.trailing_zeros() as u8).ok()
    }

    pub fn count_bits(&self) -> u32 {
        self.0.count_ones()
    }

    pub fn set_bit(&mut self, sq: Square) {
        self.0 |= 1 << sq as usize;
    }

    pub fn is_set(&self, sq: Square) -> bool {
        *self & Bitboard::from(sq) != Bitboard::EMPTY
    }

    pub fn is_empty(&self) -> bool {
        *self == Bitboard::EMPTY
    }

    pub fn intersects(&self, other: &Bitboard) -> bool {
        self.0 & other.0 != 0
    }

    pub fn from_file(file: u8) -> Bitboard {
        Bitboard::A_FILE << file as usize
    }

    pub fn from_rank(rank: u8) -> Bitboard {
        Bitboard::RANK_8 << (8 * rank as usize)
    }

    pub fn draw(&self) -> String {
        let mut bb = String::new();
        for sq in 0..64 {
            if sq % 8 == 0 {
                bb.push('\n');
            }
            if self.0 & 1 << sq == 0 {
                bb.push_str(" 0");
            } else {
                bb.push_str(" 1");
            }
        }

        bb
    }

    pub fn bits(&self) -> u64 {
        self.0
    }
}

impl Iterator for Bitboard {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        let sq = self.bitscan();
        *self &= *self - 1;
        sq
    }
}

impl From<Square> for Bitboard {
    fn from(value: Square) -> Self {
        Bitboard(1 << value as usize)
    }
}

impl BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

impl BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs
    }
}

impl BitXor for Bitboard {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl BitXorAssign for Bitboard {
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs;
    }
}

impl Shl for Bitboard {
    type Output = Self;

    fn shl(self, rhs: Self) -> Self::Output {
        Self(self.0 << rhs.0)
    }
}

impl Shl<usize> for Bitboard {
    type Output = Self;

    fn shl(self, rhs: usize) -> Self::Output {
        Self(self.0 << rhs)
    }
}

impl ShlAssign for Bitboard {
    fn shl_assign(&mut self, rhs: Self) {
        *self = *self << rhs;
    }
}

impl Shr for Bitboard {
    type Output = Self;

    fn shr(self, rhs: Self) -> Self::Output {
        Self(self.0 >> rhs.0)
    }
}

impl Shr<usize> for Bitboard {
    type Output = Self;

    fn shr(self, rhs: usize) -> Self::Output {
        Self(self.0 >> rhs)
    }
}

impl ShrAssign for Bitboard {
    fn shr_assign(&mut self, rhs: Self) {
        *self = *self >> rhs;
    }
}

impl Add for Bitboard {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Add<u64> for Bitboard {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl AddAssign for Bitboard {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for Bitboard {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0.wrapping_sub(rhs.0))
    }
}

impl Sub<u64> for Bitboard {
    type Output = Self;

    fn sub(self, rhs: u64) -> Self::Output {
        Self(self.0.wrapping_sub(rhs))
    }
}

impl SubAssign for Bitboard {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Not for Bitboard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl Mul<u64> for Bitboard {
    type Output = u64;

    fn mul(self, rhs: u64) -> Self::Output {
        self.0.overflowing_mul(rhs).0
    }
}
