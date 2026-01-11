pub mod fen;
pub mod game;
pub mod position;
pub mod state;

pub use position::{Cell, Color, Piece, Position, Square, PieceKind}; //deleted BOARD120, constant import is private, added PieceKind
pub use state::{GameState, State, Undo};
