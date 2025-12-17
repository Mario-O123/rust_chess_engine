/*
    Brettdarstellung 120er Array 
    xxxxxxxxxx    <- offboard, Board -> 21 - 98
    xxxxxxxxxx
    xxabcdefghxx
    xxabcdefghxx
    xxabcdefghxx
    xxabcdefghxx
    xxabcdefghxx
    xxabcdefghxx
    xxabcdefghxx
    xxabcdefghxx
    xxxxxxxxxx
    xxxxxxxxxx
    this is a test...
*/

pub type Mailbox120 = [i32; 120];
pub type Mailbox64 = [i32; 64];

pub const OFFBOARD: i32 = -1;
pub const EMPTY: i32 = 0;

pub const TEST:usize = 1;

pub static MAILBOX120: [i32;120] = [
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, 0, 1, 2, 3, 4, 5, 6, 7, -1, -1,
    -1, -1, 8, 9, 10, 11, 12, 13, 14, 15, -1, -1,
    -1, -1, 16, 17, 18, 19, 20, 21, 22, 23, -1, -1
    -1, -1, 24, 25, 26, 27, 28, 29, 30, 31, -1, -1,
    -1, -1, 32, 33, 34, 35, 36, 37, 38, 39, -1, -1,
    -1, -1, 40, 41, 42, 43, 44, 45, 46, 47, -1, -1,
    -1, -1, 48, 49, 50, 51, 52, 53, 54, 55, -1, -1,
    -1, -1, 56, 57, 58, 59, 60, 61, 62, 63, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
];

pub static MAILBOX64: [i32; 64] = [
    21, 22, 23, 24, 25, 26, 27, 28, 
    29, 30, 31, 32, 33, 34, 35, 36, 
    37, 38, 39, 40, 41, 42, 43, 44, 
    45, 46, 47, 48, 49, 50, 51, 52, 
    53, 54, 55, 56, 57, 58, 59, 60, 
    61, 62, 63, 64, 65, 66, 67, 67, 
    68, 69, 70, 71, 72, 73, 74, 75, 
    76, 77, 78, 79, 80, 81, 82, 83, 
];


//wandelt eine 64er Feld in ein 120er Feld um 
pub fn to_mailbox120(square64: usize) -> Option<usize> {
    MAILBOX64[square64] as usize
}

//wandelt eine 120er Feld in ein 64er Feld um 
//gibt None() zurück falls OFFBOARD
pub fn to_square64(square120:usize) -> Option<usize> {
    let square = MAILBOX120[square120];
    if square == OFFBOARD {
        None
    } else {
        Some(square as usize)
    }
}

//Prüft ob ein 120er außerhalb des Boards ist 
pub fn is_offboard(square120:usize) -> bool {
    MAILBOX120[square120] == OFFBOARD
}

//File und rank einer 64er Poition -
pub fn file_of(square64: usize) -> usize {
    square64 % 8
}

pub fn rank_of(square64: usize) -> usize {
    square64 / 8
}

