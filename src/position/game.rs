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
        gamestate.save_history(&position);

        Self {
            position,
            gamestate,
            gamestatus: GameStatus::Ongoing,
        }
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

        self.position.make_move(mv);
        self.gamestate.save_history(&self.position);
        self.gamestatus = self.compute_status();
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
