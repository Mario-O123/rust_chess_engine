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

impl GameState {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
        }
    }

    pub fn save_history(&mut self, pos: &Position) {
        self.history.push(State {
            board: pos.board,
            player_to_move: pos.player_to_move,
            en_passant_square: pos.en_passant_square,
            castling_rights: pos.castling_rights,
            zobrist: pos.zobrist,
            half_move_clock: pos.half_move_clock,
            move_counter: pos.move_counter,
            king_sq: pos.king_sq,
            piece_counter: pos.piece_counter,
        });
    }
}

// Move is not implemented yet
pub struct Undo {
    pub mv: Move,
    pub captured: Option<Piece>,
    pub prev_ep_sq: Option<Square>,
    pub prev_castling: u8,
    pub prev_zobrist: u64,
    pub prev_hm_clock: u16,
}
