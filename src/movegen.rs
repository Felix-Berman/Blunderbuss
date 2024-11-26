use crate::{
    bitboard::Bitboard,
    magic::{BISHOP_BITS, MAGICS, ROOK_BITS},
    piece::{
        Colour::{self, *},
        Piece,
    },
    position::Castling,
    square::Square,
};

pub fn king_attacks(sq: Square) -> Bitboard {
    let king = Bitboard::from(sq);
    let mut attacks = Bitboard::EMPTY;
    attacks |= king << 7 & !Bitboard::H_FILE;
    attacks |= king << 8;
    attacks |= king << 9 & !Bitboard::A_FILE;
    attacks |= king << 1 & !Bitboard::A_FILE;
    attacks |= king >> 7 & !Bitboard::A_FILE;
    attacks |= king >> 8;
    attacks |= king >> 9 & !Bitboard::H_FILE;
    attacks |= king >> 1 & !Bitboard::H_FILE;
    attacks
}

pub fn knight_attacks(sq: Square) -> Bitboard {
    let knight = Bitboard::from(sq);
    let mut attacks = Bitboard::EMPTY;
    attacks |= knight << 6 & !Bitboard::H_FILE & !Bitboard::G_FILE;
    attacks |= knight << 15 & !Bitboard::H_FILE;
    attacks |= knight << 17 & !Bitboard::A_FILE;
    attacks |= knight << 10 & !Bitboard::A_FILE & !Bitboard::B_FILE;
    attacks |= knight >> 6 & !Bitboard::A_FILE & !Bitboard::B_FILE;
    attacks |= knight >> 15 & !Bitboard::A_FILE;
    attacks |= knight >> 17 & !Bitboard::H_FILE;
    attacks |= knight >> 10 & !Bitboard::H_FILE & !Bitboard::G_FILE;

    attacks
}

pub fn pawn_attacks(sq: Square, side: Colour) -> Bitboard {
    let pawn = Bitboard::from(sq);

    match side {
        White => (pawn >> 7 & !Bitboard::A_FILE) | (pawn >> 9 & !Bitboard::H_FILE),
        Black => (pawn << 7 & !Bitboard::H_FILE) | (pawn << 9 & !Bitboard::A_FILE),
    }
}

pub fn pawn_push(sq: Square, side: Colour) -> Bitboard {
    let pawn = Bitboard::from(sq);

    match side {
        White => pawn << 8,
        Black => pawn >> 8,
    }
}

fn rook_attacks(sq: Square, mut occ: Bitboard) -> Bitboard {
    occ &= MAGICS.rook_magics[sq as usize].mask;
    let mut occ = occ * MAGICS.rook_magics[sq as usize].magic;
    occ >>= 64 - ROOK_BITS[sq];

    MAGICS.rook_attacks[sq as usize][occ as usize]
}

fn bishop_attacks(sq: Square, mut occ: Bitboard) -> Bitboard {
    occ &= MAGICS.bishop_magics[sq as usize].mask;
    let mut occ = occ * MAGICS.bishop_magics[sq as usize].magic;
    occ >>= 64 - BISHOP_BITS[sq];

    MAGICS.bishop_attacks[sq as usize][occ as usize]
}
