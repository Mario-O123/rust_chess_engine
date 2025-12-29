// === Public API ===
pub mod r#move;
pub mod pseudo_legal_movegen;
pub mod legal_move_filter;
pub mod perft;

// === Internal helpers ===
mod pawn;
mod piece;
mod pin;
mod attack;

// === Re-exports for clean imports ===
pub use r#move::{Move, MoveType, PromotionPiece};
pub use pseudo_legal_movegen::generate_pseudo_legal_moves;
pub use legal_move_filter::filter_legal_moves;
pub use perft::perft;
