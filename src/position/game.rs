use std::collections::HashMap;

use crate::board::mailbox120::SQUARE120_TO_SQUARE64;
use crate::movegen::attack::is_in_check;
use crate::movegen::legal_move_filter::filter_legal_moves;
use crate::movegen::pseudo_legal_movegen::generate_pseudo_legal_moves;
use crate::position::{Color, PieceKind, Position, State};

pub enum Gamestatus {
    Ongoing,
    Checkmate { winner: Color },
    Stalemate,
    DrawRepetition,
    DrawInsufficientMaterial,
    Draw50Moves,
}

pub struct Game {
    position: Position,
    gamestate: Vec<State>,
    gamestatus: Gamestatus,
}

impl Game {
    pub fn new() -> Self {
        Self {
            position: Position::starting_position(),
            gamestate: Vec::new(),
            gamestatus: Gamestatus::Ongoing,
        }
    }

    // half_move_clock has to reset when a piece is captured
    // or a pawn is moved
    pub fn check_draw_50_moves(&self) -> Gamestatus {
        if self.position.half_move_clock >= 100 {
            return Gamestatus::Draw50Moves;
        }
        Gamestatus::Ongoing
    }

    pub fn check_draw_insuffiecient_material(&self) -> Gamestatus {
        const INSUFFICIENT: [[u8; 12]; 5] = [
            [0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1], // WK - BK
            [0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1], // WN, WK - BK
            [0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 1], // WK - BN, BK
            [0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 1], // WB, WK - BK
            [0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 1], // WK - BB, BK
        ];

        // WB, WK - BB, BK
        // is only insufficient if both bishops are on the same square color
        const MAYBE_INSUFFICIENT: [u8; 12] = [0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1];

        for i in INSUFFICIENT {
            if self.position.piece_counter == i {
                return Gamestatus::DrawInsufficientMaterial;
            }
        }

        if self.position.piece_counter == MAYBE_INSUFFICIENT && self.bishops_same_color() {
            return Gamestatus::DrawInsufficientMaterial;
        }
        Gamestatus::Ongoing
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
            return (white_sq64 % 2) == (black_sq64 % 2);
        }
        false
    }

    // checks via zobrist hash if a position occured 3 or more times.
    // If so, the function returns a draw through repetition.
    pub fn check_draw_repetition(&self) -> Gamestatus {
        let mut position_counter: HashMap<u64, u8> = HashMap::new();

        for history in &self.gamestate {
            let count = position_counter.entry(history.zobrist).or_insert(0);
            *count += 1;
            if *count >= 3 {
                return Gamestatus::DrawRepetition;
            }
        }
        Gamestatus::Ongoing
    }

    // checks if no legal move available and if King is in check
    pub fn check_checkmate(&self) -> Gamestatus {
        let side = self.position.player_to_move;
        let pseudo = generate_pseudo_legal_moves(&self.position);
        let legal = filter_legal_moves(&self.position, &pseudo);

        if legal.is_empty() && is_in_check(self.position, side) {
            return Gamestatus::Checkmate {
                winner: side.opposite(),
            };
        } else {
            return Gamestatus::Ongoing;
        }
    }

    // checks if no legal move available and if King is NOT in check
    pub fn check_stalemate(&self) -> Gamestatus {
        let side = self.position.player_to_move;
        let pseudo = generate_pseudo_legal_moves(&self.position);
        let legal = filter_legal_moves(&self.position, &pseudo);

        if legal.is_empty() && !is_in_check(&self.position, side) {
            return Gamestatus::Stalemate;
        } else {
            return Gamestatus::Ongoing;
        }
    }
}
