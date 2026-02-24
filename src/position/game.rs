use crate::board::mailbox120::SQUARE120_TO_SQUARE64;
use crate::movegen::Move;
use crate::movegen::attack::is_in_check;
use crate::movegen::legal_move_filter::filter_legal_moves;
use crate::movegen::pseudo_legal_movegen::generate_pseudo_legal_moves;
use crate::position::{Color, GameState, PieceKind, Position};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GameStatus {
    Ongoing,
    Checkmate { winner: Color },
    Stalemate,
    DrawRepetition,
    DrawInsufficientMaterial,
    Draw50Moves,
}

pub struct Game {
    position: Position,
    gamestate: GameState,
    gamestatus: GameStatus,
}

impl Game {
    pub fn new() -> Self {
        let position = Position::starting_position();
        let mut gamestate = GameState::new();
        gamestate.reset(&position);

        Self {
            position,
            gamestate,
            gamestatus: GameStatus::Ongoing,
        }
    }
    pub fn position(&self) -> &Position {
        &self.position
    }
    pub fn position_mut(&mut self) -> &mut Position {
        &mut self.position
    }
    pub fn status(&self) -> GameStatus {
        self.gamestatus
    }
    pub fn gamestate(&self) -> &GameState {
        &self.gamestate
    }

    //checks all draw and checkmate options
    fn compute_status(&self) -> GameStatus {
        self.check_checkmate_or_stalemate()
            .or_else(|| self.check_draw_insufficient_material())
            .or_else(|| self.check_draw_repetition())
            .or_else(|| self.check_draw_50_moves())
            .unwrap_or(GameStatus::Ongoing)
    }

    pub fn try_play_move(&mut self, mv: Move) {
        if self.gamestatus != GameStatus::Ongoing {
            return;
        }

        let undo = self.position.make_move_with_undo(mv);
        self.gamestate.record_after_make(undo, &self.position);
        self.gamestatus = self.compute_status();
    }

    pub fn undo(&mut self) -> bool {
        let Some(undo) = self.gamestate.pop_undo() else {
            return false;
        };
        self.position.undo_move(undo);
        self.gamestatus = self.compute_status();
        true
    }

    // half_move_clock has to reset when a piece is captured
    // or a pawn is moved
    fn check_draw_50_moves(&self) -> Option<GameStatus> {
        if self.position.half_move_clock >= 100 {
            return Some(GameStatus::Draw50Moves);
        }
        None
    }

    fn check_draw_insufficient_material(&self) -> Option<GameStatus> {
        const INSUFFICIENT: [[u8; 12]; 7] = [
            [0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1], // WK - BK
            [0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1], // WN, WK - BK
            [0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 1], // WK - BN, BK
            [0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 1], // WB, WK - BK
            [0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 1], // WK - BB, BK
            [0, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1], // WK, WN, WN - BK
            [0, 0, 0, 0, 0, 1, 0, 2, 0, 0, 0, 1], // WK - BK, BN, BN
        ];

        // WB, WK - BB, BK
        // is only insufficient if both bishops are on the same square color
        const MAYBE_INSUFFICIENT: [u8; 12] = [0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1];

        for i in INSUFFICIENT {
            if self.position.piece_counter == i {
                return Some(GameStatus::DrawInsufficientMaterial);
            }
        }

        if self.position.piece_counter == MAYBE_INSUFFICIENT && self.bishops_same_color() {
            return Some(GameStatus::DrawInsufficientMaterial);
        }
        None
    }

    // returns true if white and black bishop have the same square color
    fn bishops_same_color(&self) -> bool {
        if let (Some(wb), Some(bb)) = (
            self.position
                .find_single_piece(Color::White, PieceKind::Bishop),
            self.position
                .find_single_piece(Color::Black, PieceKind::Bishop),
        ) {
            let white_sq64 = SQUARE120_TO_SQUARE64[wb.as_usize()] as i8;
            let black_sq64 = SQUARE120_TO_SQUARE64[bb.as_usize()] as i8;
            return Self::square_color(white_sq64) == Self::square_color(black_sq64);
        }
        false
    }

    // checks via zobrist hash if a position occured 3 or more times.
    // If so, the function returns a draw through repetition.
    fn check_draw_repetition(&self) -> Option<GameStatus> {
        let current = self.position.zobrist;
        let count = self
            .gamestate
            .history
            .iter()
            .filter(|s| s.zobrist == current)
            .count();
        if count >= 3 {
            return Some(GameStatus::DrawRepetition);
        }
        None
    }

    fn check_checkmate_or_stalemate(&self) -> Option<GameStatus> {
        let side = self.position.player_to_move;
        let check = is_in_check(&self.position, side);
        let pseudo = generate_pseudo_legal_moves(&self.position);
        let legal = filter_legal_moves(&self.position, &pseudo);

        if legal.is_empty() && !check {
            return Some(GameStatus::Stalemate);
        } else if legal.is_empty() && check {
            return Some(GameStatus::Checkmate {
                winner: side.opposite(),
            });
        } else {
            None
        }
    }

    // Helperfunction
    fn square_color(sq64: i8) -> i8 {
        let file = sq64 % 8;
        let rank = sq64 / 8;
        (file + rank) & 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::movegen::Move;
    use crate::position::{Cell, Position};

    // Helperfunction a1 = sq(0,0); h8 = sq(7,7);
    fn sq(file: i32, rank: i32) -> usize {
        (21 + file + rank * 10) as usize
    }

    // Puts a piece on a sq
    fn put(pos: &mut Position, s: usize, color: Color, kind: PieceKind) {
        pos.board[s] = Cell::Piece(crate::position::Piece { color, kind });
    }

    #[test]
    fn test_stalemate() {
        let mut pos = Position::empty();
        pos.player_to_move = Color::White;

        let a8 = sq(0, 7);
        let a7 = sq(0, 6);
        let a6 = sq(0, 5);
        let b6 = sq(1, 5);

        put(&mut pos, a8, Color::Black, PieceKind::King);
        put(&mut pos, a7, Color::White, PieceKind::Pawn);
        put(&mut pos, b6, Color::White, PieceKind::King);

        pos.king_sq = pos.compute_king_sq();
        pos.piece_counter = pos.compute_piece_counter();
        pos.zobrist = pos.compute_zobrist();

        let mut game = Game {
            position: pos,
            gamestate: GameState::new(),
            gamestatus: GameStatus::Ongoing,
        };

        game.gamestate.reset(&game.position);

        let mv = Move::new(b6, a6);
        game.try_play_move(mv);
        debug_assert_eq!(game.gamestatus, GameStatus::Stalemate);
    }

    #[test]
    fn test_checkmate() {
        let mut pos = Position::empty();
        pos.player_to_move = Color::White;

        let a8 = sq(0, 7);
        let h7 = sq(7, 6);
        let b7 = sq(1, 6);
        let b6 = sq(1, 5);

        put(&mut pos, a8, Color::Black, PieceKind::King);
        put(&mut pos, h7, Color::White, PieceKind::Queen);
        put(&mut pos, b6, Color::White, PieceKind::King);

        pos.king_sq = pos.compute_king_sq();
        pos.piece_counter = pos.compute_piece_counter();
        pos.zobrist = pos.compute_zobrist();

        let mut game = Game {
            position: pos,
            gamestate: GameState::new(),
            gamestatus: GameStatus::Ongoing,
        };

        game.gamestate.reset(&game.position);

        let mv = Move::new(h7, b7);
        game.try_play_move(mv);
        debug_assert_eq!(
            game.gamestatus,
            GameStatus::Checkmate {
                winner: (Color::White)
            }
        );
    }

    #[test]
    fn test_insufficient() {
        let mut pos = Position::empty();
        pos.player_to_move = Color::White;

        let a8 = sq(0, 7);
        let b6 = sq(7, 6);
        let f7 = sq(5, 6);
        let a2 = sq(0, 1);

        put(&mut pos, a8, Color::Black, PieceKind::King);
        put(&mut pos, f7, Color::Black, PieceKind::Rook);
        put(&mut pos, b6, Color::White, PieceKind::King);
        put(&mut pos, a2, Color::White, PieceKind::Bishop);

        pos.king_sq = pos.compute_king_sq();
        pos.piece_counter = pos.compute_piece_counter();
        pos.zobrist = pos.compute_zobrist();

        let mut game = Game {
            position: pos,
            gamestate: GameState::new(),
            gamestatus: GameStatus::Ongoing,
        };

        game.gamestate.reset(&game.position);

        let mv = Move::new(a2, f7);
        game.try_play_move(mv);
        debug_assert_eq!(game.gamestatus, GameStatus::DrawInsufficientMaterial);
    }

    #[test]
    fn test_insufficient_same_color_bishops() {
        let mut pos = Position::empty();
        pos.player_to_move = Color::White;

        let a8 = sq(0, 7);
        let b6 = sq(7, 6);
        let f7 = sq(5, 6);
        let a2 = sq(0, 1);
        let d7 = sq(3, 6);

        put(&mut pos, a8, Color::Black, PieceKind::King);
        put(&mut pos, f7, Color::Black, PieceKind::Rook);
        put(&mut pos, d7, Color::Black, PieceKind::Bishop);
        put(&mut pos, b6, Color::White, PieceKind::King);
        put(&mut pos, a2, Color::White, PieceKind::Bishop);

        pos.king_sq = pos.compute_king_sq();
        pos.piece_counter = pos.compute_piece_counter();
        pos.zobrist = pos.compute_zobrist();

        let mut game = Game {
            position: pos,
            gamestate: GameState::new(),
            gamestatus: GameStatus::Ongoing,
        };

        game.gamestate.reset(&game.position);

        let mv = Move::new(a2, f7);
        game.try_play_move(mv);
        debug_assert_eq!(game.gamestatus, GameStatus::DrawInsufficientMaterial);
    }

    #[test]
    fn test_not_insufficient_opposite_color_bishops() {
        let mut pos = Position::empty();
        pos.player_to_move = Color::White;

        let a8 = sq(0, 7);
        let b6 = sq(7, 6);
        let f7 = sq(5, 6);
        let a2 = sq(0, 1);
        let e7 = sq(4, 6);

        put(&mut pos, a8, Color::Black, PieceKind::King);
        put(&mut pos, f7, Color::Black, PieceKind::Rook);
        put(&mut pos, e7, Color::Black, PieceKind::Bishop);
        put(&mut pos, b6, Color::White, PieceKind::King);
        put(&mut pos, a2, Color::White, PieceKind::Bishop);

        pos.king_sq = pos.compute_king_sq();
        pos.piece_counter = pos.compute_piece_counter();
        pos.zobrist = pos.compute_zobrist();

        let mut game = Game {
            position: pos,
            gamestate: GameState::new(),
            gamestatus: GameStatus::Ongoing,
        };
        game.gamestate.reset(&game.position);

        let mv = Move::new(a2, f7);
        game.try_play_move(mv);
        debug_assert_eq!(game.gamestatus, GameStatus::Ongoing);
    }

    #[test]
    fn draw_50_moves() {
        let mut pos = Position::starting_position();
        pos.half_move_clock = 99;

        let mut game = Game {
            position: pos,
            gamestate: GameState::new(),
            gamestatus: GameStatus::Ongoing,
        };

        game.gamestate.reset(&game.position);

        game.gamestatus = game.compute_status();
        assert_eq!(game.gamestatus, GameStatus::Ongoing);

        game.position.half_move_clock = 100;
        game.gamestatus = game.compute_status();
        assert_eq!(game.gamestatus, GameStatus::Draw50Moves);
    }

    #[test]
    fn check_draw_repetition() {
        let g1 = sq(6, 0);
        let f3 = sq(5, 2);
        let g8 = sq(6, 7);
        let f6 = sq(5, 5);

        let mut game = Game::new();

        let mv_w1 = Move::new(g1, f3);
        let mv_w2 = Move::new(f3, g1);
        let mv_b1 = Move::new(g8, f6);
        let mv_b2 = Move::new(f6, g8);

        game.try_play_move(mv_w1);
        game.try_play_move(mv_b1);
        game.try_play_move(mv_w2);
        game.try_play_move(mv_b2);
        debug_assert_eq!(game.gamestatus, GameStatus::Ongoing);

        game.try_play_move(mv_w1);
        game.try_play_move(mv_b1);
        game.try_play_move(mv_w2);
        game.try_play_move(mv_b2);
        debug_assert_eq!(game.gamestatus, GameStatus::DrawRepetition);
    }
}
