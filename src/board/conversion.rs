//! this module contains small helper functions around:
//! -converting mailbox120 indices to file/rank,
//! -parsing algebraic square notation,
//! -converting between piece codes and characters

use super::mailbox120::{
    BOARD_SIZE, OFFBOARD, SQUARE64_TO_SQUARE120, SQUARE120_TO_SQUARE64, is_on_board,
    square120_from_file_rank,
};

///extracts (file, rank) from a mailbox120 square index
/// 
///# preconditions:
/// -square120 < BOARD_SIZE
/// -SQUARE120_TO_SQUARE64[square120] != OFFBOARD
/// these are enforced via debug_assert! in debug builds
#[inline]
pub fn file_rank_from_square120(square120: usize) -> (u8, u8) {
    debug_assert!(square120 < BOARD_SIZE);
    debug_assert!(SQUARE120_TO_SQUARE64[square120] != OFFBOARD);

    let adjusted = square120 - 21;
    let file = (adjusted % 10) as u8;
    let rank = (adjusted / 10) as u8;
    (file, rank)
}

//Algebraic notation of square120 (example: "e4")
#[inline]
pub fn square120_to_string(square120: usize) -> Option<String> {
    if !is_on_board(square120) {
        return None;
    }
    let (file, rank) = file_rank_from_square120(square120);
    let file_char = (b'a' + file) as char;
    let rank_char = (b'1' + rank) as char;
    Some(format!("{}{}", file_char, rank_char))
}

///parses algebraic notation (eg "A1", "e4") into a mailbox120 square index, returns "None" in invalid cases
pub fn square120_from_string(s: &str) -> Option<usize> {
    let b = s.as_bytes();
    if b.len() != 2 {
        return None;
    }

    let file = match b[0] {
        b'a'..=b'h' => b[0],
        b'A'..=b'H' => b[0] + 32, //note for "+32": ASCII notation for lowercasing
        _ => return None,
    };

    let rank = match b[1] {
        b'1'..=b'8' => b[1],
        _ => return None,
    };

    let file_idx = (file - b'a') as usize;
    let rank_idx = (rank - b'1') as usize;

    let square120 = square120_from_file_rank(file_idx, rank_idx);
    if is_on_board(square120) {
        Some(square120)
    } else {
        None
    }
}

//Convert Piece to Char
//White = positiv + Uppercase, Black =  negativ + Lowercase
#[inline]
pub fn piece_to_char(piece: i8) -> char {
    match piece {
        1 => 'P',
        2 => 'N',
        3 => 'B',
        4 => 'R',
        5 => 'Q',
        6 => 'K',
        -1 => 'p',
        -2 => 'n',
        -3 => 'b',
        -4 => 'r',
        -5 => 'q',
        -6 => 'k',
        _ => '.',
    }
}

///converts an ASCII piece character into the internal piece code
///supported inputs: "P N B R Q K" (white) and "p n b r q k" (black)
pub fn char_to_piece(c: char) -> Option<i8> {
    match c {
        'P' => Some(1),
        'N' => Some(2),
        'B' => Some(3),
        'R' => Some(4),
        'Q' => Some(5),
        'K' => Some(6),
        'p' => Some(-1),
        'n' => Some(-2),
        'b' => Some(-3),
        'r' => Some(-4),
        'q' => Some(-5),
        'k' => Some(-6),
        _ => None,
    }
}

//piece to char unicode
//White -> outlines, Black -> full
#[inline]
pub fn piece_to_char_unicode(p: i8) -> char {
    match p {
        1 => '♙',
        2 => '♘',
        3 => '♗',
        4 => '♖',
        5 => '♕',
        6 => '♔',
        -1 => '♟',
        -2 => '♞',
        -3 => '♝',
        -4 => '♜',
        -5 => '♛',
        -6 => '♚',
        _ => '·',
    }
}

///converts a mailbox120 index to a square64 index, uses a safer API
/// 
/// # note: we have many instances where we simply use the mailbox120.rs mapping SQUARE120_TO_SQUARE64
/// instead of using this wrapper, in a more thorough clear-up one of the variants should
/// probably be used as the standard across all files
#[inline]
pub fn square120_to_square64(square120: usize) -> Option<usize> {
    if square120 >= BOARD_SIZE {
        return None;
    }

    let square64 = SQUARE120_TO_SQUARE64[square120];
    if square64 == OFFBOARD {
        None
    } else {
        Some(square64 as usize)
    }
}

///converts a square64 index to a mailbox120 index, uses a safer API
/// 
/// # note: same case as above
#[inline]
pub fn square64_to_square120(square64: usize) -> Option<usize> {
    if square64 >= 64 {
        None
    } else {
        Some(SQUARE64_TO_SQUARE120[square64])
    }
}

#[cfg(test)]
mod tests {
    //! unit tests for conversion helpers and invariants
    use super::*;

    ///file_rank_from_square120 has to be consistent with square120_from_file_rank
    #[test]
    fn file_rank_from_square120_matches_square120_from_file_rank() {
        for file in 0..8 {
            for rank in 0..8 {
                let square120 = square120_from_file_rank(file, rank);
                let (f, r) = file_rank_from_square120(square120);
                assert_eq!(f as usize, file);
                assert_eq!(r as usize, rank);
            }
        }
    }

    #[test]
    fn square120_from_string_accepts_valid_lowercase() {
        assert!(square120_from_string("a1").is_some());
        assert!(square120_from_string("h8").is_some());
        assert!(square120_from_string("e4").is_some());
    }

    #[test]
    fn square120_from_string_accepts_valid_uppercase() {
        assert_eq!(square120_from_string("A1"), square120_from_string("a1"));
        assert_eq!(square120_from_string("H8"), square120_from_string("h8"));
        assert_eq!(square120_from_string("E4"), square120_from_string("e4"));
    }

    /// # note: the array of string slices for this test was created using a LLM
    #[test]
    fn square120_from_string_rejects_invalid_inputs() {
        for s in ["", "a", "a0", "a9", "aa", "A0", "a56", "i1", "11"] {
            assert_eq!(square120_from_string(s), None, "should reject: {s}");
        }
    }

    ///round-trip square120-> string-> square120 must preserve the square
    #[test]
    fn square120_to_string_and_back_returns_same_square120() {
        for file in 0..8 {
            for rank in 0..8 {
                let square120 = square120_from_file_rank(file, rank);

                let s = square120_to_string(square120).expect("valid square must create a string");
                let back = square120_from_string(&s).expect("string mus parse back");

                assert_eq!(
                    square120, back,
                    "converting back failed for {file}, {rank} -> {s}"
                );
            }
        }
    }

    ///round-trip: square64-> square120-> square64 must preserve the index
    #[test]
    fn square64_to_120_and_back_returns_same_square64() {
        for square64 in 0..64 {
            let square120 =
                square64_to_square120(square64).expect("sqquare64 must convert to square120");
            let back =
                square120_to_square64(square120).expect("square120 must convert back to square64");
            assert_eq!(back, square64);
        }
    }

    #[test]
    fn square120_to_square64_rejects_offboard_and_out_of_bounds() {
        assert_eq!(square120_to_square64(BOARD_SIZE), None);
        assert_eq!(square120_to_square64(BOARD_SIZE + 1), None);

        assert_eq!(square120_to_square64(0), None);
        assert_eq!(square120_to_square64(119), None);
    }
}
