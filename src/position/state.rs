//! game state snapshots and hinary helpers
//! modules provides lightweight [`State`] snapshots of [`Position`] and [`GameState`]
//! container that tracks position history plus an undo stack

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
    ///creates a [`State`] snapshot from the given [`Position`]
    ///used by [`GameState`] to store history entries efficiently
    ///the snapshot is a plain data copy of the relevant fields
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

///stores game history and undo information
///
#[derive(Debug, Default)]
pub struct GameState {
    pub history: Vec<State>,
    //Undo-stack for CLI-Undo
    pub undo_stack: Vec<Undo>,
}

impl GameState {
    /// Creates an empty [`GameState`]
    pub fn new() -> Self {
        Self::default()
    }

    ///pushes a snapshot of `pos` onto the history list
    ///this does not modify the undo stack
    //moved logic to fn from_position to use frequently
    pub fn save_history(&mut self, pos: &Position) {
        self.history.push(State::from_position(pos));
    }

    ///clears history and undo stack, then records the given position as the initial state
    ///call this once at game start (so `history` is not empty)
    pub fn reset(&mut self, pos: &Position) {
        self.history.clear();
        self.undo_stack.clear();
        self.save_history(pos);
    }

    ///records the result of making a move:
    ///- pushes `undo` onto the undo stack
    ///- pushes a snapshot of the resulting position onto the history
    ///intended to be called right after `make_move_with_undo`
    pub fn record_after_make(&mut self, undo: Undo, pos_after: &Position) {
        self.undo_stack.push(undo);
        self.save_history(pos_after);
    }

    ///pops one undo record and rewinds history by one step.
    ///returns:
    ///  Some(undo) if a move can be undone
    ///  None       if there is no move to undo (only the initial state exists)
    ///
    /// # Note
    /// this updates the stacks only. The caller must apply the returned [`Undo`]
    /// (or otherwise restore the position) to actually revert the current position
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
