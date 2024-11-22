use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl,
    ShlAssign, Shr, ShrAssign, Sub, SubAssign,
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Square(usize);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Bitboard(u64);

impl Bitboard {
    pub const EMPTY: Bitboard = Bitboard(0);

    pub fn bitscan(self) -> Square {
        Square(self.0.trailing_zeros() as usize)
    }
}

impl Iterator for Bitboard {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        let sq = self.bitscan();
        if sq.0 > 63 {
            return None;
        }
        self.0 &= !(1 << sq.0);
        Some(sq)
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
        *self = Self(self.0 & rhs.0);
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
        *self = Self(self.0 | rhs.0)
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
        *self = Self(self.0 ^ rhs.0);
    }
}

impl Shl for Bitboard {
    type Output = Self;

    fn shl(self, rhs: Self) -> Self::Output {
        Self(self.0 << rhs.0)
    }
}

impl ShlAssign for Bitboard {
    fn shl_assign(&mut self, rhs: Self) {
        *self = Self(self.0 << rhs.0);
    }
}

impl Shr for Bitboard {
    type Output = Self;

    fn shr(self, rhs: Self) -> Self::Output {
        Self(self.0 >> rhs.0)
    }
}

impl ShrAssign for Bitboard {
    fn shr_assign(&mut self, rhs: Self) {
        *self = Self(self.0 >> rhs.0);
    }
}

impl Add for Bitboard {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for Bitboard {
    fn add_assign(&mut self, rhs: Self) {
        *self = Self(self.0 + rhs.0);
    }
}

impl Sub for Bitboard {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for Bitboard {
    fn sub_assign(&mut self, rhs: Self) {
        *self = Self(self.0 - rhs.0);
    }
}

impl Not for Bitboard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}
