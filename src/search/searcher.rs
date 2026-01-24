use std::time::Instant;

use crate::evaluation::Evaluator;
use crate::movegen::{
    Move, generate_legal_captures_in_place, generate_legal_moves_in_place, is_in_check,
};
use crate::position::{Cell, Color, PieceKind, Position};

const INF: i32 = 50000;
const MATE: i32 = 30_000;

#[derive(Clone, Copy)]
pub struct SearchLimits {
    pub max_depth: u8,
    pub max_nodes: Option<u64>,
    pub max_time_ms: Option<u64>,
}

pub struct SearchResult {
    pub best_move: Move,
    pub score_cp: i32,
    pub depth: u8,
    pub nodes: u64,
}

pub struct Searcher<E: Evaluator> {
    eval: E,
    nodes: u64,
    start: Instant,
    limits: SearchLimits,
    history: Vec<u64>,
    move_buf: Vec<Move>,
}

impl<E: Evaluator> Searcher<E> {
    pub fn new(eval: E) -> Self {
        Self {
            eval,
            nodes: 0,
            start: Instant::now(),
            limits: SearchLimits {
                max_depth: 1,
                max_nodes: None,
                max_time_ms: None,
            },
            history: Vec::new(),
            move_buf: Vec::new(),
        }
    }

    pub fn search(&mut self, pos: &mut Position, limits: SearchLimits) -> SearchResult {
        self.limits = limits;
        self.nodes = 0;
        self.start = Instant::now();

        self.history.clear();
        self.history.push(pos.zobrist);

        if pos.half_move_clock >= 100 {
            return SearchResult {
                best_move: Move::NULL,
                score_cp: 0,
                depth: 0,
                nodes: 0,
            };
        }

        let mut best_move = Move::NULL;
        let mut best_score = 0;
        let mut reached_depth = 0;

        for d in 1..=self.limits.max_depth {
            let (mv, sc) = self.root(pos, d as i32);

            best_move = mv;
            best_score = sc;
            reached_depth = d;

            if mv.is_null() {
                break;
            }

            if self.should_stop() {
                break;
            }
        }
        SearchResult {
            best_move,
            score_cp: best_score,
            depth: reached_depth,
            nodes: self.nodes,
        }
    }

    fn root(&mut self, pos: &mut Position, depth: i32) -> (Move, i32) {
        self.move_buf.clear();
        generate_legal_moves_in_place(pos, &mut self.move_buf);

        if self.move_buf.is_empty() {
            return (Move::NULL, self.terminal_score(pos, 0));
        }

        //simple ordering
        //self.move_buf.sort_by_key(|&m| -Self::move_order_score(pos, m));

        let mut best_mv = Move::NULL;
        let mut best = -INF;
        let mut alpha = -INF;
        let beta = INF;

        let mut scored_moves: Vec<(Move, i32)> = self
            .move_buf
            .iter()
            .map(|&m| (m, Self::move_order_score(pos, m)))
            .collect();

        scored_moves.sort_by_key(|&(_, score)| -score);

        for (mv, _) in scored_moves {
            let undo = pos.make_move_with_undo(mv);
            self.history.push(pos.zobrist);

            let score = -self.negamax(pos, depth - 1, 1, -beta, -alpha);

            self.history.pop();
            pos.undo_move(undo);

            if self.should_stop() {
                break;
            }

            if score > best {
                best = score;
                best_mv = mv;
            }
            if score > alpha {
                alpha = score;
            }
        }
        (best_mv, best)
    }

    fn negamax(
        &mut self,
        pos: &mut Position,
        depth: i32,
        ply: i32,
        mut alpha: i32,
        beta: i32,
    ) -> i32 {
        self.nodes += 1;
        if self.should_stop() {
            return alpha;
        }

        //draw rules
        if pos.half_move_clock >= 100 {
            return 0;
        }
        if self.is_repetition(pos.zobrist) {
            return 0;
        }
        if depth <= 0 {
            return self.quiescence(pos, ply, alpha, beta);
        }
        self.move_buf.clear();
        generate_legal_moves_in_place(pos, &mut self.move_buf);

        if self.move_buf.is_empty() {
            return self.terminal_score(pos, ply);
        }

        //self.move_buf.sort_by_key(|&m| -Self::move_order_score(pos, m));

        let mut scored_moves: Vec<(Move, i32)> = self
            .move_buf
            .iter()
            .map(|&m| (m, Self::move_order_score(pos, m)))
            .collect();

        scored_moves.sort_by_key(|&(_, score)| -score);

        for (mv, _) in scored_moves {
            let undo = pos.make_move_with_undo(mv);
            self.history.push(pos.zobrist);

            let score = -self.negamax(pos, depth - 1, ply + 1, -beta, -alpha);

            self.history.pop();
            pos.undo_move(undo);

            if score > alpha {
                alpha = score;
            }
            if alpha >= beta {
                break;
            }
            if self.should_stop() {
                break;
            }
        }
        alpha
    }

    fn quiescence(&mut self, pos: &mut Position, ply: i32, mut alpha: i32, beta: i32) -> i32 {
        self.nodes += 1;
        if self.should_stop() {
            return alpha;
        }

        //if in check also allow evasion not only captures
        let stm = pos.player_to_move;
        if is_in_check(pos, stm) {
            self.move_buf.clear();
            generate_legal_moves_in_place(pos, &mut self.move_buf);

            if self.move_buf.is_empty() {
                return -MATE + ply as i32;
            }

            //self.move_buf.sort_by_key(|&m| -Self::move_order_score(pos, m));

            let mut scored_moves: Vec<(Move, i32)> = self
                .move_buf
                .iter()
                .map(|&m| (m, Self::move_order_score(pos, m)))
                .collect();

            scored_moves.sort_by_key(|&(_, score)| -score);

            for (mv, _) in scored_moves {
                let undo = pos.make_move_with_undo(mv);
                self.history.push(pos.zobrist);

                let score = -self.quiescence(pos, ply + 1, -beta, -alpha);

                self.history.pop();
                pos.undo_move(undo);

                if score > alpha {
                    alpha = score;
                }
                if alpha >= beta {
                    return beta;
                }
                if self.should_stop() {
                    break;
                }
            }
            return alpha;
        }

        //stand-pat
        let stand_pat = self.eval_stm(pos);
        if stand_pat >= beta {
            return beta;
        }
        if stand_pat > alpha {
            alpha = stand_pat;
        }
        self.move_buf.clear();
        generate_legal_captures_in_place(pos, &mut self.move_buf);
        //self.move_buf.sort_by_key(|&m| -Self::move_order_score(pos, m));
        let mut scored_moves: Vec<(Move, i32)> = self
            .move_buf
            .iter()
            .map(|&m| (m, Self::move_order_score(pos, m)))
            .collect();

        scored_moves.sort_by_key(|&(_, score)| -score);
        for (mv, _) in scored_moves {
            let undo = pos.make_move_with_undo(mv);
            self.history.push(pos.zobrist);

            let score = -self.quiescence(pos, ply + 1, -beta, -alpha);

            self.history.pop();
            pos.undo_move(undo);

            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
            }
            if self.should_stop() {
                break;
            }
        }
        alpha
    }

    fn terminal_score(&mut self, pos: &Position, ply: i32) -> i32 {
        let stm = pos.player_to_move;
        if is_in_check(pos, stm) {
            -MATE + ply
        } else {
            0
        }
    }

    #[inline]
    fn eval_stm(&mut self, pos: &Position) -> i32 {
        let s = self.eval.evaluate(pos);
        if pos.player_to_move == Color::White {
            s
        } else {
            -s
        }
    }

    #[inline]
    fn is_repetition(&self, key: u64) -> bool {
        self.history.iter().rev().skip(1).any(|&k| k == key)
    }

    fn should_stop(&self) -> bool {
        if let Some(n) = self.limits.max_nodes {
            if self.nodes >= n {
                return true;
            }
        }
        if let Some(ms) = self.limits.max_time_ms {
            if self.start.elapsed().as_millis() as u64 >= ms {
                return true;
            }
        }
        false
    }

    //ordering helpers
    #[inline]
    fn piece_value(kind: PieceKind) -> i32 {
        match kind {
            PieceKind::Pawn => 100,
            PieceKind::Knight => 320,
            PieceKind::Bishop => 330,
            PieceKind::Rook => 500,
            PieceKind::Queen => 900,
            PieceKind::King => 20000,
        }
    }

    #[inline]
    fn move_order_score(pos: &Position, mv: Move) -> i32 {
        let mut s = 0;

        if mv.is_promotion() {
            s += 90000;

            if let Cell::Piece(victim) = pos.board[mv.to_sq()] {
                s += Self::piece_value(victim.kind);
            }
            return s;
        }

        if mv.is_en_passant() {
            if let Cell::Piece(att) = pos.board[mv.from_sq()] {
                s += 10000 + 100 - Self::piece_value(att.kind);
            } else {
                s += 10000;
            }
        } else if let Cell::Piece(victim) = pos.board[mv.to_sq()] {
            if let Cell::Piece(att) = pos.board[mv.from_sq()] {
                s += 10000 + Self::piece_value(victim.kind) - Self::piece_value(att.kind);
            } else {
                s += 10000 + Self::piece_value(victim.kind);
            }
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluation::classical::ClassicalEval;
    use crate::position::Position;

    // Test 1: Basic functionality
    #[test]
    fn test_search_finds_legal_move() {
        let mut pos = Position::starting_position();
        let eval = ClassicalEval::new();
        let mut searcher = Searcher::new(eval);

        let limits = SearchLimits {
            max_depth: 3,
            max_nodes: None,
            max_time_ms: None,
        };

        let result = searcher.search(&mut pos, limits);

        assert!(!result.best_move.is_null());
        assert_eq!(result.depth, 3);
        assert!(result.nodes > 0);
    }

    // Test 2: Matt detection in 1 (Scholar's Mate Setup)
    #[test]
    fn test_finds_mate_in_one() {
        // Position: White#s turn can mate with Qf7#
        let fen = "r1bqkb1r/pppp1Qpp/2n2n2/4p3/2B1P3/8/PPPP1PPP/RNB1K1NR b KQkq - 0 4";
        let mut pos = Position::from_fen(fen).unwrap();
        let eval = ClassicalEval::new();
        let mut searcher = Searcher::new(eval);

        // Für Schwarz - sollte Matt-Score erkennen
        let limits = SearchLimits {
            max_depth: 2,
            max_nodes: None,
            max_time_ms: None,
        };

        let result = searcher.search(&mut pos, limits);

        // Score should be negativ  (Matt vs Black)
        assert!(result.score_cp < -25000);
    }

    // Test 3: Stalemate check
    #[test]
    fn test_recognizes_stalemate() {
        let fen = "7k/5Q2/6K1/8/8/8/8/8 b - - 0 1";
        let mut pos = Position::from_fen(fen).unwrap();
        let eval = ClassicalEval::new();
        let mut searcher = Searcher::new(eval);

        let limits = SearchLimits {
            max_depth: 1,
            max_nodes: None,
            max_time_ms: None,
        };

        let result = searcher.search(&mut pos, limits);

        assert!(result.best_move.is_null());
        assert_eq!(result.score_cp, 0);
    }

    // Test 4: 50-Move Rule
    #[test]
    fn test_fifty_move_rule() {
        let mut pos = Position::starting_position();
        pos.half_move_clock = 100; // 50-move rule detected

        let eval = ClassicalEval::new();
        let mut searcher = Searcher::new(eval);

        let limits = SearchLimits {
            max_depth: 3,
            max_nodes: None,
            max_time_ms: None,
        };

        let result = searcher.search(&mut pos, limits);

        assert_eq!(result.score_cp, 0);
    }

    // Test 5: Repetition Detection
    #[test]
    fn test_repetition_detection() {
        let pos = Position::starting_position();
        let eval = ClassicalEval::new();
        let mut searcher = Searcher::new(eval);

        searcher.history.push(pos.zobrist);
        searcher.history.push(12345);
        searcher.history.push(pos.zobrist); // Repetition!

        assert!(searcher.is_repetition(pos.zobrist));
    }

    // Test 6: Respect Time Limit
    #[test]
    fn test_respects_time_limit() {
        let mut pos = Position::starting_position();
        let eval = ClassicalEval::new();
        let mut searcher = Searcher::new(eval);

        let limits = SearchLimits {
            max_depth: 100,
            max_nodes: None,
            max_time_ms: Some(100),
        };

        let start = std::time::Instant::now();
        let result = searcher.search(&mut pos, limits);
        let elapsed = start.elapsed().as_millis();

        // should stop ~100-200ms
        assert!(elapsed < 300);
        assert!(result.depth < 100);
    }

    // Test 7: Respekt Node Limit
    #[test]
    fn test_respects_node_limit() {
        let mut pos = Position::starting_position();
        let eval = ClassicalEval::new();
        let mut searcher = Searcher::new(eval);

        let limits = SearchLimits {
            max_depth: 100,
            max_nodes: Some(1000), // Only 1000 Nodes
            max_time_ms: None,
        };

        let result = searcher.search(&mut pos, limits);

        assert!(result.nodes <= 1100);
        assert!(result.depth < 100);
    }

    // Test 8: Captures > Quiet Moves
    #[test]
    fn test_move_ordering_prefers_captures() {
        let fen = "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 2";
        let pos = Position::from_fen(fen).unwrap();

        // Mailbox 120:
        // d2 = 34 (Rank 2, file d)
        // d3 = 44 (Rank 3, file d)
        // e4 = 55 (Rank 4, file e)
        // e5 = 65 (Rank 5, file e)

        let quiet_move = Move::new(34, 44); // d2-d3
        let capture_move = Move::new(55, 65); // e4xe5

        let quiet_score = Searcher::<ClassicalEval>::move_order_score(&pos, quiet_move);
        let capture_score = Searcher::<ClassicalEval>::move_order_score(&pos, capture_move);

        println!("Quiet score: {}", quiet_score);
        println!("Capture score: {}", capture_score);

        assert!(capture_score > quiet_score);
    }

    // Test 9: Promotions > everything
    #[test]
    fn test_move_ordering_prefers_promotions() {
        let fen = "4k3/P7/8/8/8/8/8/4K3 w - - 0 1";
        let pos = Position::from_fen(fen).unwrap();

        use crate::movegen::PromotionPiece;

        // a7 = 81, a8 = 91
        // e1 = 25, e2 = 35
        let promotion = Move::new_promotion(81, 91, PromotionPiece::Queen);
        let normal = Move::new(25, 35); // e1-e2

        let promo_score = Searcher::<ClassicalEval>::move_order_score(&pos, promotion);
        let normal_score = Searcher::<ClassicalEval>::move_order_score(&pos, normal);

        println!("Promo score: {}", promo_score);
        println!("Normal score: {}", normal_score);

        assert!(promo_score >= 90000);
        assert!(promo_score > normal_score);
    }

    // Test 10: Reached Depth ist correct with early Stop
    #[test]
    fn test_reached_depth_correct_on_early_stop() {
        let mut pos = Position::starting_position();
        let eval = ClassicalEval::new();
        let mut searcher = Searcher::new(eval);

        let limits = SearchLimits {
            max_depth: 10,
            max_nodes: Some(100),
            max_time_ms: None,
        };

        let result = searcher.search(&mut pos, limits);

        assert!(result.depth < 10);

        assert!(result.nodes <= 150);
    }

    // Test 11
    #[test]
    fn test_position_unchanged_after_search() {
        let mut pos = Position::starting_position();
        let original_zobrist = pos.zobrist;
        let original_player = pos.player_to_move;

        let eval = ClassicalEval::new();
        let mut searcher = Searcher::new(eval);

        let limits = SearchLimits {
            max_depth: 3,
            max_nodes: Some(5000),
            max_time_ms: None,
        };

        let result = searcher.search(&mut pos, limits);

        println!("Searched {} nodes", result.nodes);
        println!(
            "Original zobrist: {}, After: {}",
            original_zobrist, pos.zobrist
        );

        assert_eq!(pos.zobrist, original_zobrist);
        assert_eq!(pos.player_to_move, original_player);
    }

    //test 12
    #[test]
    fn test_detects_checkmate() {
        let fen = "6k1/5ppp/8/8/8/8/5PPP/4r1K1 w - - 0 1";
        let mut pos = Position::from_fen(fen).unwrap();

        let white_in_check = is_in_check(&pos, Color::White);
        println!("White in check: {}", white_in_check);

        let mut moves = Vec::new();
        generate_legal_moves_in_place(&mut pos, &mut moves);
        println!("Legal moves: {}", moves.len());

        if white_in_check && moves.is_empty() {
            println!("This is CHECKMATE");
        } else if !white_in_check && moves.is_empty() {
            println!("This is STALEMATE");
        }

        let eval = ClassicalEval::new();
        let mut searcher = Searcher::new(eval);

        let limits = SearchLimits {
            max_depth: 1,
            max_nodes: None,
            max_time_ms: None,
        };

        let result = searcher.search(&mut pos, limits);

        println!("Score: {}", result.score_cp);
        println!("Best move: {:?}", result.best_move);

        if white_in_check && moves.is_empty() {
            assert!(
                result.score_cp <= -25000,
                "Should detect mate, got {}",
                result.score_cp
            );
        }
    }
}
