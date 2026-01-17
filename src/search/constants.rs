//Search Constants

pub const MAX_DEPTH: u8 = 64;
pub const INFINITY: i32 = 50000;
pub const MATE_SCORE: i32 = 49000;

#[inline]
pub fn mate_in_N(ply: i32) -> i32 {
    MATE_SCORE - ply
}

#[inline]
pub fn mated_in_N(ply: i32) -> i32 {
    -MATE_SCORE + ply
}

#[inline]
pub fn is_mate_score(score: i32) -> bool {
    score.abs() >= MATE_SCORE - MAX_DEPTH as i32
}
