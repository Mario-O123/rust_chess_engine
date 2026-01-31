mod pst;
use super::Evaluator;
use crate::board::mailbox120::SQUARE120_TO_SQUARE64;
use crate::position::{Cell, Color, Piece, PieceKind, Position};
use pst::*;

pub struct ClassicalEval;

// Values in Centipawns
const PAWN_VALUE: i32 = 100;
const KNIGHT_VALUE: i32 = 320;
const BISHOP_VALUE: i32 = 330;
const ROOK_VALUE: i32 = 500;
const QUEEN_VALUE: i32 = 900;
const KING_VALUE: i32 = 0;
const PLAYERS_TURN: i32 = 10;
const BISHOP_PAIR: i32 = 30;
const PHASE_MAX: i32 = 24;
const CASTLING_BONUS: i32 = 30;
const UNDEVELOPED_PENALTY: i32 = -10;

impl ClassicalEval {
    pub fn new() -> Self {
        Self
    }

    fn get_piece_value(piece: &Piece) -> i32 {
        match piece.kind {
            PieceKind::Pawn => PAWN_VALUE,
            PieceKind::Knight => KNIGHT_VALUE,
            PieceKind::Bishop => BISHOP_VALUE,
            PieceKind::Rook => ROOK_VALUE,
            PieceKind::Queen => QUEEN_VALUE,
            PieceKind::King => KING_VALUE,
        }
    }

    fn mirror_sq64(sq64: usize) -> usize {
        let file = sq64 % 8;
        let rank = sq64 / 8;
        (7 - rank) * 8 + file
    }

    fn phase_calculator(piece: &Piece) -> i32 {
        match piece.kind {
            PieceKind::Knight => 1,
            PieceKind::Bishop => 1,
            PieceKind::Rook => 2,
            PieceKind::Queen => 4,
            _ => 0,
        }
    }

    fn king_pst_blend(sq64: usize, phase: i32) -> i32 {
        (PST_KING_MG[sq64] * phase + PST_KING_EG[sq64] * (PHASE_MAX - phase)) / PHASE_MAX
    }

    fn get_square_value(sq: usize, piece: &Piece) -> i32 {
        // check if sq64_i8 is valid
        let sq64_i8 = SQUARE120_TO_SQUARE64[sq];
        if sq64_i8 < 0 {
            return 0;
        }

        let mut sq64 = sq64_i8 as usize;
        if piece.color == Color::Black {
            sq64 = Self::mirror_sq64(sq64);
        }

        match piece.kind {
            PieceKind::Pawn => PST_PAWN[sq64],
            PieceKind::Knight => PST_KNIGHT[sq64],
            PieceKind::Bishop => PST_BISHOP[sq64],
            PieceKind::Rook => PST_ROOK[sq64],
            PieceKind::Queen => PST_QUEEN[sq64],
            _ => 0,
        }
    }

    #[inline]
    fn bitmask(to_check: u8, mask: u8) -> bool {
        to_check & mask != 1
    }
}

impl Evaluator for ClassicalEval {
    fn evaluate(&mut self, pos: &Position) -> i32 {
        let mut score = 0;
        let mut bishop_counter_white = 0;
        let mut bishop_counter_black = 0;
        let mut phase_counter = 0;
        let mut white_king_sq64: usize = 64;
        let mut black_king_sq64: usize = 64;
        let mut white_undeveloped_count = 0;
        let mut black_undeveloped_count = 0;

        // Bonus for piece and square depending on PST
        for (sq, cell) in pos.board.iter().enumerate() {
            if let Cell::Piece(piece) = cell {
                let value = Self::get_piece_value(piece) + Self::get_square_value(sq, piece);

                match piece.color {
                    Color::White => score += value,
                    Color::Black => score -= value,
                };

                if piece.kind == PieceKind::Bishop {
                    match piece.color {
                        Color::White => bishop_counter_white += 1,
                        Color::Black => bishop_counter_black += 1,
                    };
                    let bsq_i8 = SQUARE120_TO_SQUARE64[sq];
                    if bsq_i8 >= 0 {
                        let bsq = bsq_i8 as usize;
                        match piece.color {
                            Color::White => {
                                if bsq == 2 {
                                    white_undeveloped_count += 1;
                                }
                                if bsq == 5 {
                                    white_undeveloped_count += 1;
                                }
                            }
                            Color::Black => {
                                if Self::mirror_sq64(bsq) == 2 {
                                    black_undeveloped_count += 1;
                                }
                                if Self::mirror_sq64(bsq) == 5 {
                                    black_undeveloped_count += 1;
                                }
                            }
                        }
                    }
                };

                if piece.kind == PieceKind::King {
                    let ksq_i8 = SQUARE120_TO_SQUARE64[sq];
                    if ksq_i8 >= 0 {
                        let ksq = ksq_i8 as usize;
                        match piece.color {
                            Color::White => white_king_sq64 = ksq,
                            Color::Black => black_king_sq64 = Self::mirror_sq64(ksq),
                        }
                    }
                }

                if piece.kind == PieceKind::Knight {
                    let nsq_i8 = SQUARE120_TO_SQUARE64[sq];
                    if nsq_i8 >= 0 {
                        let nsq = nsq_i8 as usize;
                        match piece.color {
                            Color::White => {
                                if nsq == 1 {
                                    white_undeveloped_count += 1;
                                }
                                if nsq == 6 {
                                    white_undeveloped_count += 1;
                                }
                            }
                            Color::Black => {
                                if Self::mirror_sq64(nsq) == 1 {
                                    black_undeveloped_count += 1;
                                }
                                if Self::mirror_sq64(nsq) == 6 {
                                    black_undeveloped_count += 1;
                                }
                            }
                        }
                    }
                }

                phase_counter += Self::phase_calculator(piece);
            }
        }

        let white_king_sq64 = match white_king_sq64 < 64 {
            true => white_king_sq64,
            false => {
                debug_assert!(false, "missing king(s)");
                return 0;
            }
        };
        let black_king_sq64 = match black_king_sq64 < 64 {
            true => black_king_sq64,
            false => {
                debug_assert!(false, "missing king(s)");
                return 0;
            }
        };

        // The 2 PSTs are blend, depending on non-pawn-pieces on board
        let phase = phase_counter.clamp(0, PHASE_MAX);
        score += Self::king_pst_blend(white_king_sq64, phase);
        score -= Self::king_pst_blend(black_king_sq64, phase);

        // Bonus for castling in early game
        let opening_bonus = (CASTLING_BONUS * phase) / PHASE_MAX;
        let white_castled = (white_king_sq64 == 2 && Self::bitmask(0b0010, pos.castling_rights))
            || (white_king_sq64 == 6 && Self::bitmask(0b0001, pos.castling_rights));
        let black_castled_sq = (black_king_sq64 == 2 && Self::bitmask(0b1000, pos.castling_rights))
            || (black_king_sq64 == 6 && Self::bitmask(0b0100, pos.castling_rights));

        if white_castled {
            score += opening_bonus;
        }
        if black_castled_sq {
            score -= opening_bonus;
        }

        let undevelopment_penalty = (UNDEVELOPED_PENALTY * phase) / PHASE_MAX;
        score += undevelopment_penalty * white_undeveloped_count;
        score -= undevelopment_penalty * black_undeveloped_count;

        // Bonus for bishop pair
        if bishop_counter_white >= 2 {
            score += BISHOP_PAIR;
        };
        if bishop_counter_black >= 2 {
            score -= BISHOP_PAIR;
        }

        match pos.player_to_move {
            Color::White => score += PLAYERS_TURN,
            Color::Black => score -= PLAYERS_TURN,
        };

        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sq(file: i32, rank: i32) -> usize {
        (21 + file + rank * 10) as usize
    }

    // Puts a piece on a sq
    fn put(pos: &mut Position, s: usize, color: Color, kind: PieceKind) {
        pos.board[s] = Cell::Piece(crate::position::Piece { color, kind });
    }

    #[test]
    fn eval_starting_position() {
        let pos = Position::starting_position();
        let mut class_eval = ClassicalEval::new();
        println!("Starting position eval: {}", class_eval.evaluate(&pos));
        assert_eq!(class_eval.evaluate(&pos), 10);
    }

    // Should be 10 because of Bonus for Players Turn
    #[test]
    fn eval_winning_position_black() {
        let mut pos = Position::empty();
        let mut class_eval = ClassicalEval::new();

        let a1 = sq(0, 0);
        let h2 = sq(7, 1);
        let h8 = sq(7, 7);

        put(&mut pos, a1, Color::White, PieceKind::King);
        put(&mut pos, h2, Color::Black, PieceKind::Queen);
        put(&mut pos, h8, Color::Black, PieceKind::King);

        println!("Winning Position Black eval: {}", class_eval.evaluate(&pos));
        assert!(class_eval.evaluate(&pos) < 0);
    }

    // Debug Test
    #[test]
    fn debug_test() {
        let mut pos = Position::starting_position();
        let mut class_eval = ClassicalEval::new();

        let e4 = sq(4, 3);
        let e5 = sq(4, 4);
        let a5 = sq(0, 4);

        put(&mut pos, e4, Color::White, PieceKind::Pawn);
        put(&mut pos, a5, Color::Black, PieceKind::Pawn);

        println!("Eval after e4 e5 eval: {}", class_eval.evaluate(&pos));
    }

    #[test]
    fn eval_winning_position_white() {
        let mut pos = Position::empty();
        let mut class_eval = ClassicalEval::new();

        let b1 = sq(1, 0);
        let h4 = sq(7, 3);
        let h5 = sq(7, 4);

        put(&mut pos, b1, Color::Black, PieceKind::King);
        put(&mut pos, h4, Color::White, PieceKind::Rook);
        put(&mut pos, h5, Color::White, PieceKind::King);

        println!("Winning position white eval: {}", class_eval.evaluate(&pos));
        assert!(class_eval.evaluate(&pos) > 0);
    }

    #[test]
    fn center_knight_is_better() {
        let mut pos = Position::empty();
        let mut class_eval = ClassicalEval::new();
        let a1 = sq(0, 0);
        let h8 = sq(7, 7);
        let a8 = sq(7, 0);
        let e4 = sq(4, 3);

        put(&mut pos, a1, Color::Black, PieceKind::King);
        put(&mut pos, h8, Color::White, PieceKind::King);
        put(&mut pos, e4, Color::White, PieceKind::Knight);
        put(&mut pos, a8, Color::Black, PieceKind::Knight);

        println!("CenterKnight eval: {}", class_eval.evaluate(&pos));
        assert!(class_eval.evaluate(&pos) > 0);
    }
}
