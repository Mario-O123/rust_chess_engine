// === Public API ===
pub mod legal_move_filter;
pub mod r#move;
pub mod perft;
pub mod pseudo_legal_movegen;

// === Internal helpers ===
mod attack;
mod pawn;
mod piece;

// === Re-exports for clean imports ===
pub use legal_move_filter::filter_legal_moves;
pub use r#move::{Move, MoveType, PromotionPiece};
pub use perft::perft;
pub use pseudo_legal_movegen::generate_pseudo_legal_moves;
