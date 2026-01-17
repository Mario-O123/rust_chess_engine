use crate::board::mailbox120::BOARD_SIZE as BOARD120;
use crate::movegen::Move;
use crate::position::{Cell, Color, Piece, Position, Square};

// Order vor king_sq: WK, BK
// Order for piece_counter: WP, WN, WB, WR, WQ, WK, BP, BN, BB, BR, BQ, BK
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
#[derive(Clone, Debug)]
pub struct Undo {
    pub mv: Move,
    pub moving_piece: Piece,

    //capture info
    pub captured: Option<Piece>,
    pub captured_sq: Option<usize>, //only for en-passant

    //castling info
    pub rook_from: Option<usize>,
    pub rook_to: Option<usize>,

    //previous state snapshot
    pub prev_player_to_move: Color,
    pub prev_ep_sq: Option<Square>,
    pub prev_castling: u8,
    pub prev_zobrist: u64,
    pub prev_hm_clock: u16,
    pub prev_move_counter: u16,
    pub prev_king_sq: [u8; 2],
    pub prev_piece_counter: [u8; 12],
}
