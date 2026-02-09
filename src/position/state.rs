use crate::board::mailbox120::BOARD_SIZE as BOARD120;
use crate::movegen::Move;
use crate::position::{Cell, Color, Piece, Position, Square};

// Order vor king_sq: WK, BK
// Order for piece_counter: WP, WN, WB, WR, WQ, WK, BP, BN, BB, BR, BQ, BK
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

impl State {
    #[inline]
    pub fn from_position(pos: &Position) -> Self {
        Self {
            board: pos.board,
            player_to_move: pos.player_to_move,
            en_passant_square: pos.en_passant_square,
            castling_rights: pos.castling_rights,
            zobrist: pos.zobrist,
            half_move_clock: pos.half_move_clock,
            move_counter: pos.move_counter,
            king_sq: pos.king_sq,
            piece_counter: pos.piece_counter,
        }
    }
}

#[derive(Debug, Default)]
pub struct GameState {
    pub history: Vec<State>,
    //Undo-stack for CLI-Undo
    pub undo_stack: Vec<Undo>,
}

impl GameState {
    pub fn new() -> Self {
        Self::default()
    }

    //moved logic to fn from_position to use frequently
    pub fn save_history(&mut self, pos: &Position) {
        self.history.push(State::from_position(pos));
    }

    //call one time at start, so history isn't empty
    pub fn reset(&mut self, pos: &Position) {
        self.history.clear();
        self.undo_stack.clear();
        self.save_history(pos);
    }

    //call after make_move_with_undo: Undo + new Position in the history
    pub fn record_after_make(&mut self, undo: Undo, pos_after: &Position) {
        self.undo_stack.push(undo);
        self.save_history(pos_after);
    }

    //expected: history contains at least the starting Position
    pub fn pop_undo(&mut self) -> Option<Undo> {
        if self.history.len() <= 1 {
            return None;
        }
        self.history.pop(); // remove current position snapshot
        self.undo_stack.pop()
    }
}

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
