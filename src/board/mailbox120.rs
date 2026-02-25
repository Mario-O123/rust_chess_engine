//! mailbox120 board representation
//! In this module, we represent the board in mailbox120 (10*12 indices),
//! here we place the 8*8 board in a 10*12 grid and surround it with a guard ring
//! of squares which are all marked OFFBOARD, this makes for quick offboard checks before a move can succeed

use std::sync::LazyLock;

///number of squares within the 10*12 board
pub const BOARD_SIZE: usize = 120;
///value we use in lookup tables to check if an index is within the 8*8 board
pub const OFFBOARD: i8 = -1;

///move-offsets for rook directions (+1/-1 = right/left, +10/-10 = up/down)
pub const ROOK_DIRECTIONS: [i8; 4] = [1, -1, 10, -10];
///move-offsets for bishop directions (diagonals)
pub const BISHOP_DIRECTIONS: [i8; 4] = [9, -9, 11, -11];
///move-offsets for queen directions (rook+bishop directions)
pub const QUEEN_DIRECTIONS: [i8; 8] = [1, -1, 9, -9, 10, -10, 11, -11];
///move-offsets for knight jumps
pub const KNIGHT_DIRECTIONS: [i8; 8] = [8, -8, 12, -12, 19, -19, 21, -21];

///helper fn: compute the mailbox120 index from "file" and "rank"
///the real 8*8 board starts at offset 21
///# preconditions: "file" and "rank" must each be in 0..8
pub const fn square120_from_file_rank(file: usize, rank: usize) -> usize {
    21 + file + (rank * 10)
}

///returns "true" if "square120" refers to a valid square on the real 8*8 board
pub fn is_on_board(square120: usize) -> bool {
    square120 < BOARD_SIZE && SQUARE120_TO_SQUARE64[square120] != OFFBOARD
}

///lookup table: mailbox120 -> square64
///real board squares map to "0..64", padding squares map to [`OFFBOARD`]
pub static SQUARE120_TO_SQUARE64: LazyLock<[i8; BOARD_SIZE]> = LazyLock::new(|| {
    let mut lookup = [OFFBOARD; BOARD_SIZE];
    let mut square64: usize = 0;

    for rank in 0..8 {
        for file in 0..8 {
            let square120: usize = square120_from_file_rank(file, rank);
            lookup[square120] = square64 as i8;
            square64 += 1;
        }
    }

    lookup
});

///lookup table: square64 -> mailbox120
///for each square "0..63", stores the corresponding mailbox index
pub static SQUARE64_TO_SQUARE120: LazyLock<[usize; 64]> = LazyLock::new(|| {
    let mut lookup = [0usize; 64];
    let mut square64: usize = 0;

    for rank in 0..8 {
        for file in 0..8 {
            let square120 = square120_from_file_rank(file, rank);
            lookup[square64] = square120;
            square64 += 1;
        }
    }
    lookup
});

#[cfg(test)]
mod tests {
    //! unit tests for mailbox120 mappings and invariants
    //! we want [`is_on_board`] to reject guard-border indices and accept real board swuares
    //! [`SQUARE64_TO_SQUARE120`] and [`SQUARE120_TO_SQUARE64`] should be consistent
    use super::*;

    ///verifies that [`is_on_board`] is safe on boundary values,
    ///it must reject obvious offbaord indices and accept known valid board squares
    #[test]
    fn is_on_board_is_safe() {
        assert!(!is_on_board(0));
        assert!(!is_on_board(119));
        assert!(is_on_board(square120_from_file_rank(0, 0)));
        assert!(is_on_board(square120_from_file_rank(7, 7)));
    }

    ///round trip invariant: for every square64 in 0..64,
    ///mappingsquare64 -> square120 -> square64 must return the original index,
    ///if we claim that square64 and square120 are 2 representations of the same sauare, then converting there and back (64->120->64) should
    ///return the original value for all squares, it would catch an inconsistent mapping between the 2 lookup tables and wrong table entries
    #[test]
    fn lookup_square64_to_square120_and_back_works() {
        for square64 in 0..64usize {
            let square120 = SQUARE64_TO_SQUARE120[square64];
            assert!(is_on_board(square120));
            assert_eq!(SQUARE120_TO_SQUARE64[square120], square64 as i8);
        }
    }
}
