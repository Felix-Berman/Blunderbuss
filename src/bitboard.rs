use std::{fmt::{Display, Formatter}, ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Deref, DerefMut, Index, IndexMut, Not, Shl, ShlAssign, Shr, ShrAssign}};

use num::{Integer, FromPrimitive};
use num_derive::{FromPrimitive, ToPrimitive};

#[derive(FromPrimitive, ToPrimitive, Clone, Copy, PartialEq, PartialOrd, Debug)]
pub enum Square {
    A8,  B8,  C8,  D8,  E8,  F8,  G8,  H8, 
    A7,  B7,  C7,  D7,  E7,  F7,  G7,  H7, 
    A6,  B6,  C6,  D6,  E6,  F6,  G6,  H6, 
    A5,  B5,  C5,  D5,  E5,  F5,  G5,  H5, 
    A4,  B4,  C4,  D4,  E4,  F4,  G4,  H4, 
    A3,  B3,  C3,  D3,  E3,  F3,  G3,  H3, 
    A2,  B2,  C2,  D2,  E2,  F2,  G2,  H2, 
    A1,  B1,  C1,  D1,  E1,  F1,  G1,  H1,
}

impl Square {
    pub fn add(self, i: i8) -> Option<Square> {
        Square::from_i8((self as i8) + i)
    }

    pub fn rank(&self) -> i8 {
        *self as i8 / 8
    }

    pub fn file(&self) -> i8 {
        *self as i8 % 8
    }

    pub fn from_algebraic(algebraic: &str) -> Option<Square> {
        if algebraic.len() != 2 {
            return None;
        }
        let file = algebraic.chars().nth(0)?.to_ascii_lowercase();
        let rank = algebraic.chars().nth(1)?;
        if !('a'..='h').contains(&file) || !('1'..='8').contains(&rank) {
            return None;
        }
        let file = file as u8 - b'a';
        let rank = 8 - (rank as u8 - b'0');
        Some(Square::from_u8(rank * 8 + file).unwrap())
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let rank = *self as u8 / 8;
        let file = *self as u8 % 8;
        write!(f, "{}{}",(file + b'a') as char, (8 - rank))
    }
}

impl<T> Index<Square> for [T; 64] {
    type Output = T;
    fn index(&self, sq: Square) -> &Self::Output {
        let sq_idx = sq as usize;
        &self[sq_idx]
    }
}

impl<T> IndexMut<Square> for [T; 64] {
    fn index_mut(&mut self, sq: Square) -> &mut Self::Output {
        let sq_idx = sq as usize;
        &mut self[sq_idx]
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Bitboard(pub u64);

impl Bitboard {
    pub const A_FILE: Bitboard = Bitboard(0x0101010101010101);
    pub const B_FILE: Bitboard = Bitboard(0x0202020202020202);
    pub const G_FILE: Bitboard = Bitboard(0x4040404040404040);
    pub const H_FILE: Bitboard = Bitboard(0x8080808080808080);

    pub fn from(sq: Square) -> Self {
        Bitboard(1 << sq as u64)
    }

    pub fn set(&mut self, sq: Square) {
        self.0 |= 1 << sq as u64;
    }

    pub fn set_masked(&mut self, sq: Square, mask: Bitboard) {
        self.0 |= mask.0 & (1 << sq as u64);
    }

    pub fn reset(&mut self, sq: Square) {
        self.0 &= !(1 << sq as u64);
    }

    pub fn is_set(&self, sq: Square) -> bool {
        (self.0 & (1 << sq as u64)) != 0
    }

    pub fn count_bits(&self) -> u32 {
        self.0.count_ones()
    }

    pub fn get_lsb(&self) -> Option<Square> {
        Square::from_u32(self.0.trailing_zeros())
    }

    pub fn intersects(&self, other: Bitboard) -> bool {
        (self.0 & other.0) != 0
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }
}

impl Iterator for Bitboard {
    type Item = Square;
    fn next(&mut self) -> Option<Self::Item> {
        let sq = self.get_lsb();
        if let Some(sq) = sq {
            self.reset(sq);
        }
        sq
    }
}

impl Deref for Bitboard {
    type Target = u64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Bitboard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }   
}

impl Display for Bitboard {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        let mut sq = 0;
        for rank in (1..=8).rev() {
            write!(f, "{} ", rank)?;
            for _ in 0..8 {
                if let Some(sq) = Square::from_i8(sq) {
                    if self.is_set(sq) {
                        write!(f, " #")?;
                    } else {
                        write!(f, " .")?;
                    }
                }
                sq += 1;
            }
            writeln!(f)?;
        }
        write!(f, "  ")?;
        for file in 'A'..='H' {
            write!(f, " {}", file)?;
        }

        writeln!(f)?;
        writeln!(f, "hex: {:#X}", self.0)?;
        writeln!(f, "bin: {:#b}", self.0)?;
        writeln!(f, "dec: {}", self.0)?;

        Ok(())
    }
}

impl Not for Bitboard {
    type Output = Self;
    fn not(self) -> Self::Output {
        Bitboard(self.0.not())
    }
}

impl BitAnd for Bitboard {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 & rhs.0)
    }
}

impl BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitOr for Bitboard {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 | rhs.0)
    }
}

impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitXor for Bitboard {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 ^ rhs.0)
    }
}

impl BitXorAssign for Bitboard {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl<T> Shr<T> for Bitboard 
where
    T: Integer,
    u64: Shr<T, Output = u64>,
{
    type Output = Self;
    fn shr(self, rhs: T) -> Self::Output {
        Bitboard(self.0 >> rhs)
    }
}

impl<T> Shl<T> for Bitboard 
where
    T: Integer,
    u64: Shl<T, Output = u64>,
{
    type Output = Self;
    fn shl(self, rhs: T) -> Self::Output {
        Bitboard(self.0 << rhs)
    }
}

impl<T> ShrAssign<T> for Bitboard 
where 
    T: Integer,
    u64: ShrAssign<T>,
{
    fn shr_assign(&mut self, rhs: T) {
        self.0 >>= rhs;
    }
}

impl<T> ShlAssign<T> for Bitboard 
where 
    T: Integer,
    u64: ShlAssign<T>,
{
    fn shl_assign(&mut self, rhs: T) {
        self.0 <<= rhs;
    }
}