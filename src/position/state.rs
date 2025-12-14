use crate::position::{BOARD120, Cell, Color, Piece, Position, Square};

pub struct State {
    pub board: [Cell; BOARD120],
    pub player_to_move: Color,
    pub en_passant_square: Option<Square>,
    pub castling_rights: u8,
    pub zobrist: u64,
    pub half_move_clock: u16,
    pub move_counter: u16,
    pub king_sq: [u8; 2],
    pub piece_counter: [u8; 12],
}

pub struct GameState {
    pub history: Vec<State>,
}

// Move is not implemented yet
pub struct Undo {
    pub mv: Move,
    pub captuared: Piece,
    pub prev_ep_sq: Option<Square>,
    pub prev_castling: u8,
    pub prev_zobrist: u64,
    pub prev_hm_clock: u16,
}
