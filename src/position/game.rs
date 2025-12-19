use std::collections::HashMap;

use crate::PieceKind;
use crate::board::mailbox120::SQUARE120_TO_SQUARE64;
use crate::movegen::{attackers_of_square, find_king, is_in_check, is_square_attacked};
use crate::position::{Cell, Color, Position, State};

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

    /// TO-DO: check if piece can move in between, so the King is not in check anymore
    // for that find all squares between the Attacker (only Slider) and King and check
    // if there is a legal move for that
    pub fn check_checkmate(&self) -> Gamestatus {
        const KING_DIRECTIONS: [i8; 8] = [1, -1, 9, -9, 10, -10, 11, -11];
        let color_in_check: Color;

        // checks if the king is in check and sets color_in_check
        if is_in_check(&self.position, Color::White) {
            color_in_check = Color::White
        } else if is_in_check(&self.position, Color::Black) {
            color_in_check = Color::Black
        } else {
            return Gamestatus::Ongoing;
        }

        let king_in_check = find_king(&self.position, color_in_check);

        let Some(king) = king_in_check else {
            return Gamestatus::Ongoing;
        };

        // checks if it is possible to take the attacking piece(s)
        // TO-DO check if the attacking piece is pinned

        let king_attackers = attackers_of_square(&self.position, king, color_in_check.opposite());

        for attacker in king_attackers {
            if is_square_attacked(&self.position, attacker, color_in_check) {
                return Gamestatus::Ongoing;
            }
        }

        // checks in all directions if the king can move:
        // checks if the adjecant squares are in check OR
        // if the adjecant squares are occupied with pieces of king´s color
        for offset in KING_DIRECTIONS {
            let king_possible_move = king + offset as usize;

            if king_possible_move < 0 || king_possible_move > 119 {
                continue;
            }

            let occupied_by_own_piece =
                if let Cell::Piece(piece) = self.position.board[king_possible_move] {
                    if piece.color == color_in_check {
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };

            if !(is_square_attacked(
                &self.position,
                king_possible_move,
                color_in_check.opposite(),
            ) || occupied_by_own_piece)
            {
                return Gamestatus::Ongoing;
            }
        }
        Gamestatus::Checkmate {
            winner: color_in_check.opposite(),
        }
    }

    // TO-DO: Implement
    pub fn check_stalemate(&self) -> Gamestatus {
        Gamestatus::Stalemate
    }
}
