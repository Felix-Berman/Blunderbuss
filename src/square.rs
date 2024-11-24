use std::ops::{Add, AddAssign, Index, IndexMut, Sub};

use num_enum::{TryFromPrimitive, TryFromPrimitiveError};

#[rustfmt::skip]
#[derive(PartialEq, Eq, PartialOrd, Ord, TryFromPrimitive, Clone, Copy)]
#[repr(u8)]
pub enum Square {
    A8, B8, C8, D8, E8, F8, G8, H8,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A1, B1, C1, D1, E1, F1, G1, H1,
}

impl Square {
    pub fn rank(&self) -> u8 {
        *self as u8 / 8
    }

    pub fn file(&self) -> u8 {
        *self as u8 % 8
    }
}

impl From<Square> for usize {
    fn from(value: Square) -> Self {
        value as usize
    }
}

impl Add<u8> for Square {
    type Output = Result<Square, TryFromPrimitiveError<Square>>;

    fn add(self, rhs: u8) -> Self::Output {
        Square::try_from_primitive(self as u8 + rhs)
    }
}

// summation mod 64 (wrapping)
impl AddAssign<u8> for Square {
    fn add_assign(&mut self, rhs: u8) {
        *self = Square::try_from_primitive((*self as u8 + rhs) % 64).unwrap();
    }
}

impl Add<i8> for Square {
    type Output = Result<Square, TryFromPrimitiveError<Square>>;

    fn add(self, rhs: i8) -> Self::Output {
        Square::try_from_primitive((self as i8 + rhs) as u8)
    }
}

impl Sub<u8> for Square {
    type Output = Result<Square, TryFromPrimitiveError<Square>>;

    fn sub(self, rhs: u8) -> Self::Output {
        Square::try_from_primitive((self as u8).wrapping_sub(rhs))
    }
}

impl TryFrom<(u8, u8)> for Square {
    type Error = TryFromPrimitiveError<Square>;

    fn try_from(ij: (u8, u8)) -> Result<Self, Self::Error> {
        Square::try_from_primitive(ij.0 * 8 + ij.1)
    }
}

impl TryFrom<&str> for Square {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut iter = value.chars();

        let file = iter.next().ok_or("Missing rank")? as u8 - b'a';
        let rank = b'8' - iter.next().ok_or("Missing file")? as u8;

        Square::try_from((rank, file)).map_err(|_| "Invalid rank or file".to_string())
    }
}

impl<T> Index<Square> for [T; 64] {
    type Output = T;

    fn index(&self, index: Square) -> &Self::Output {
        &self[usize::from(index)]
    }
}

impl<T> IndexMut<Square> for [T; 64] {
    fn index_mut(&mut self, index: Square) -> &mut Self::Output {
        &mut self[usize::from(index)]
    }
}
