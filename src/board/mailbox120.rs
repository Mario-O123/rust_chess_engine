use std::sync::LazyLock;

pub const BOARD_SIZE: usize = 120;
pub const OFFBOARD: i8 = -1;

//Directions/Movement offsets, handle King and Pawn separately
pub const ROOK_DIRECTIONS: [i8; 4] = [1, -1, 10, -10];
pub const BISHOP_DIRECTIONS: [i8; 4] = [9, -9, 11, -11];
pub const QUEEN_DIRECTIONS: [i8; 8] = [1, -1, 9, -9, 10, -10, 11, -11];
pub const KNIGHT_DIRECTIONS: [i8; 8] = [8, -8, 12, -12, 19, -19, 21, -21];

//used in the static mapping as helper
//important to always call with arguments 0..8
pub const fn square120_from_file_rank(file: usize, rank: usize) -> usize {
    21 + file + (rank * 10)
}

//this will be used later and get called many times in movegen, so inline
pub fn is_on_board(square120: usize) -> bool {
    square120 < BOARD_SIZE && SQUARE120_TO_SQUARE64[square120] != OFFBOARD
}

//the actual mappings for reading from and into the internal engine:
//here, we need i8 for the offboard markings
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
    use super::*;

    #[test]
    fn is_on_board_is_safe() {
        assert!(!is_on_board(0));
        assert!(!is_on_board(119));
        assert!(is_on_board(square120_from_file_rank(0, 0)));
        assert!(is_on_board(square120_from_file_rank(7, 7)));
    }

    #[test]
    fn lookup_square64_to_square120_and_back_works() {
        for square64 in 0..64usize {
            let square120 = SQUARE64_TO_SQUARE120[square64];
            assert!(is_on_board(square120));
            assert_eq!(SQUARE120_TO_SQUARE64[square120], square64 as i8);
        }
    }
}
