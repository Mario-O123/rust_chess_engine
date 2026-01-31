use std::time::Instant;

use crate::evaluation::Evaluator;
use crate::movegen::{
    Move, generate_legal_captures_in_place, generate_pseudo_legal_moves_in_place, is_in_check,
};
use crate::position::{Cell, Color, PieceKind, Position};
use super::tt::{Bound, TranspositionTable};

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
    tt: TranspositionTable,
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
            tt: TranspositionTable::new_mb(64),
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

        #[cfg(debug_assertions)]
        {
            debug_assert_eq!(
                pos.king_sq,
                pos.compute_king_sq(),
                "search(): king_sq invalid at entry, fen={}",
                pos.to_fen()
            );
        }

        let mut best_move = Move::NULL;
        let mut best_score = 0;
        let mut reached_depth = 0;

        for d in 1..=self.limits.max_depth {
            if self.should_stop() {
                break;
            }
            let (mv, sc, complete) = self.root(pos, d as i32);

            if !complete {
                break;
            }

            if mv.is_null() {
                if best_move.is_null() {
                    best_score = sc;
                    reached_depth = d;
                }
                break;
            }

            best_move = mv;
            best_score = sc;
            reached_depth = d;

            if self.should_stop() {
                break;
            }

            #[cfg(debug_assertions)]
            {
                debug_assert_eq!(
                    pos.king_sq,
                    pos.compute_king_sq(),
                    "search(): king_sq invalid at entry, fen={}",
                    pos.to_fen()
                );
            }
        }
        SearchResult {
            best_move,
            score_cp: best_score,
            depth: reached_depth,
            nodes: self.nodes,
        }
    }

    fn root(&mut self, pos: &mut Position, depth: i32) -> (Move, i32, bool) {
        const TT_BONUS: i32 = 1_000_000;
        
        let side_to_move = pos.player_to_move;
        let root_key = pos.zobrist;

        let tt_best = self.tt.probe(root_key).map(|e| e.best).unwrap_or(Move::NULL);

        self.move_buf.clear();
        generate_pseudo_legal_moves_in_place(pos, &mut self.move_buf);
        let mut complete = true;

        if self.move_buf.is_empty() {
            let sc = self.terminal_score(pos, 0);
            self.tt.store(root_key, depth, Self::to_tt_score(sc, 0), Bound::Exact, Move::NULL);
            return (Move::NULL, sc, complete);
        }

        // Build + sort ordered move list (captures/promos first via move_order_score)
        let mut scored_moves: Vec<(Move, i32)> = self
            .move_buf
            .iter()
            .map(|&m| {
                let tt_bonus = if m == tt_best { TT_BONUS } else { 0 };
                (m, Self::move_order_score(pos, m) + tt_bonus)
            })
            .collect();

        scored_moves.sort_by_key(|&(_, s)| -s);
        // let scored_moves_clone = scored_moves.clone();

        let mut best_mv = Move::NULL;
        let mut alpha = -INF;
        let beta = INF;

        let mut first = true;
        let mut any_legal = false;

        for (mv, _) in scored_moves {
            if self.should_stop() {
                complete = false;
                break;
            }

            let undo = pos.make_move_with_undo(mv);

            //check legality
            if is_in_check(pos, side_to_move) {
                pos.undo_move(undo);
                continue;
            }
            any_legal = true;

            self.history.push(pos.zobrist);

            // - First move: full window
            // - Others: null-window search; if it improves alpha, re-search full window
            let score = if first {
                first = false;
                -self.negamax(pos, depth - 1, 1, -beta, -alpha)
            } else {
                // Null-window probe: in negamax the child window is (-alpha-1, -alpha)
                let mut s = -self.negamax(pos, depth - 1, 1, -alpha - 1, -alpha);

                if s > alpha {
                    // Re-search with full window to get exact score
                    s = -self.negamax(pos, depth - 1, 1, -beta, -alpha);
                }
                s
            };

            self.history.pop();
            pos.undo_move(undo);

            if score > alpha {
                alpha = score;
                best_mv = mv;
            }
        }

        if !any_legal {
            let sc = self.terminal_score(pos, 0);
            if complete {
                self.tt.store(root_key, depth, Self::to_tt_score(sc, 0), Bound::Exact, Move::NULL);
            }
            return  (Move::NULL, sc, complete);
        }

        //root is an exact result if complete
        if complete {
            self.tt.store(root_key, depth, Self::to_tt_score(alpha, 0), Bound::Exact, best_mv);   
        }

        (best_mv, alpha, complete)
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
            return self.eval_stm(pos);
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

        let key = pos.zobrist;
        let orig_alpha = alpha;
        let mut beta = beta;

        //TT probe (bestmove + possible cutoff)
        let mut tt_best = Move::NULL;
        if let Some(entry) = self.tt.probe(key) {
            tt_best = entry.best;

            if (entry.depth as i32) >= depth {
                let tt_score = Self::from_tt_score(entry.score, ply);

                match entry.bound {
                    Bound::Exact => return tt_score,
                    Bound::Lower => {
                        if tt_score > alpha {
                            alpha = tt_score;
                        }
                    }
                    Bound::Upper => {
                        if tt_score > beta {
                            beta = tt_score;
                        }
                    }
                }

                if alpha >= beta {
                    return tt_score;
                }
            }
        }
        let side_to_move = pos.player_to_move;

        self.move_buf.clear();
        generate_pseudo_legal_moves_in_place(pos, &mut self.move_buf);

        if self.move_buf.is_empty() {
            let s = self.terminal_score(pos, ply);

            self.tt.store(key, depth, Self::to_tt_score(s, ply), Bound::Exact, Move::NULL);
            return s;
        }

        //self.move_buf.sort_by_key(|&m| -Self::move_order_score(pos, m));

        let mut scored_moves: Vec<(Move, i32)> = self
            .move_buf
            .iter()
            .map(|&m| {
                let tt_bonus = if m == tt_best { 1_000_000 } else {0 };
                (m, Self::move_order_score(pos, m) + tt_bonus)
            }).collect();

        scored_moves.sort_by_key(|&(_, score)| -score);

        let mut any_legal = false;
        let mut best_mv = Move::NULL;
        let mut aborted = false;

        for (mv, _) in scored_moves {
            if self.should_stop() {
                aborted = true;
                break;
            }
            let undo = pos.make_move_with_undo(mv);

            //check legality inline, instead of filtering legal before
            if is_in_check(pos, side_to_move) {
                pos.undo_move(undo);
                continue;
            }
            any_legal = true;

            self.history.push(pos.zobrist);

            let score = -self.negamax(pos, depth - 1, ply + 1, -beta, -alpha);

            self.history.pop();
            pos.undo_move(undo);

            if score > alpha {
                alpha = score;
                best_mv = mv;
            }
            if alpha >= beta {
                break;
            }
        }

        if !any_legal {
            let s = self.terminal_score(pos, ply);
            self.tt.store(key, depth, Self::to_tt_score(s, ply), Bound::Exact, Move::NULL);
            return s;
        }
        //TT store (only if not aborted by time/nodes)
        if !aborted {
            let bound = if alpha <= orig_alpha {
                Bound::Upper
            } else if alpha >= beta {
                Bound::Lower
            } else {
                Bound::Exact
            };
            self.tt.store(key, depth, Self::to_tt_score(alpha, ply), bound, best_mv);
        }
        alpha
    }

    fn quiescence(&mut self, pos: &mut Position, ply: i32, mut alpha: i32, beta: i32) -> i32 {
        self.nodes += 1;
        if self.should_stop() {
            return self.eval_stm(pos);
        }

        //if in check also allow evasion not only captures
        let side_to_move = pos.player_to_move;
        if is_in_check(pos, side_to_move) {
            self.move_buf.clear();
            generate_pseudo_legal_moves_in_place(pos, &mut self.move_buf);

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

            let mut any_legal = false;

            for (mv, _) in scored_moves {
                if self.should_stop() {
                    break;
                }

                let undo = pos.make_move_with_undo(mv);

                //legality check (did we leave the king in check?)
                if is_in_check(pos, side_to_move) {
                    pos.undo_move(undo);
                    continue;
                }
                any_legal = true;

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
                
            }

            if !any_legal {
                return -MATE + ply;
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
        generate_pseudo_legal_moves_in_place(pos, &mut self.move_buf);
        //self.move_buf.sort_by_key(|&m| -Self::move_order_score(pos, m));
        let mut scored_moves: Vec<(Move, i32)> = self
            .move_buf
            .iter()
            .filter(|&&m| {
                m.is_promotion()
                    || m.is_en_passant()
                    || matches!(pos.board[m.to_sq()], Cell::Piece(_))
            })
            .map(|&m| (m, Self::move_order_score(pos, m)))
            .collect();

        scored_moves.sort_by_key(|&(_, score)| -score);
        for (mv, _) in scored_moves {
            if self.should_stop() {
                break;
            }

            let undo = pos.make_move_with_undo(mv);

            //legality check
            if is_in_check(pos, side_to_move) {
                pos.undo_move(undo);
                continue;
            }

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

    fn is_mate_score(score: i32) -> bool {
        score.abs() > MATE - 1000
    }

    fn to_tt_score(score: i32, ply: i32) -> i32 {
        if score > MATE - 1000 {
            score + ply
        } else if score < -MATE + 1000 {
            score - ply
        } else {
            score
        }
    }

    fn from_tt_score(score: i32, ply: i32) -> i32 {
        if score > MATE - 1000 {
            score - ply
        } else if score > -MATE + 1000 {
            score + ply
        } else {
            score
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
        // Engine prefers double pawn push of central pawns
        if mv.is_double_pawn_push() {
            let file = (mv.to_sq() as i32 - 21) % 10;
            // d/e-file
            if file == 3 || file == 4 {
                s += 40;
            }
            // a/h-file
            else if file == 0 || file == 7 {
                s -= 20;
            } // a/h-file
        }

        if let Cell::Piece(p) = pos.board[mv.from_sq()] {
            if p.kind == PieceKind::Knight || p.kind == PieceKind::Bishop {
                s += 30;
            }
            if p.kind == PieceKind::Rook {
                s -= 30;
            }
        }

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
    use crate::movegen::generate_legal_moves_in_place;

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

    #[test]
    fn test_tt_does_not_change_result_fixed_depth() {
    use crate::search::tt::TranspositionTable;

    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let mut pos_a = Position::from_fen(fen).unwrap();
    let mut pos_b = Position::from_fen(fen).unwrap();

    let mut s_no_tt = Searcher::new(ClassicalEval::new());
    s_no_tt.tt = TranspositionTable::disabled();

    let mut s_tt = Searcher::new(ClassicalEval::new());
    s_tt.tt = TranspositionTable::new_mb(8);

    let limits = SearchLimits { max_depth: 4, max_nodes: None, max_time_ms: None };

    let r1 = s_no_tt.search(&mut pos_a, limits);
    let r2 = s_tt.search(&mut pos_b, limits);

    assert_eq!(r1.best_move, r2.best_move);
    assert_eq!(r1.score_cp, r2.score_cp);
    assert_eq!(r1.depth, r2.depth);
}
}

#[cfg(test)]
mod mate_score_tests {
    use super::*;

    struct DummyEval;
    impl Evaluator for DummyEval {
        fn evaluate(&mut self, pos: &Position) -> i32 {
            0
        }
    }

    #[test]
    fn tt_score_roundtrip_non_mate_is_unchanged() {
        let ply = 7;
        let score = 123; //normal eval score

        let stored = Searcher::<DummyEval>::to_tt_score(score, ply);
        let loaded = Searcher::<DummyEval>::from_tt_score(stored, ply);

        assert_eq!(stored, score);
        assert_eq!(loaded, score);
        assert!(!Searcher::<DummyEval>::is_mate_score(score));
    }

    #[test]
    fn tt_score_roundtrip_positive_mate_is_ply_neutral() {
        let ply = 9;
        let score = MATE - ply;

        let stored = Searcher::<DummyEval>::to_tt_score(score, ply);
        let loaded = Searcher::<DummyEval>::from_tt_score(stored, ply);

        assert_eq!(stored, MATE);
        assert_eq!(loaded, score);
        assert!(Searcher::<DummyEval>::is_mate_score(score));
    }

     #[test]
    fn tt_score_roundtrip_negative_mate_is_ply_neutral() {
        let ply = 6;
        let score = -MATE + ply;

        let stored = Searcher::<DummyEval>::to_tt_score(score, ply);
        let loaded = Searcher::<DummyEval>::from_tt_score(stored, ply);

        assert_eq!(stored, -MATE);
        assert_eq!(loaded, score);
        assert!(Searcher::<DummyEval>::is_mate_score(score));
    }

    #[test]
    fn is_mate_score_has_buffer_and_does_not_trigger_on_large_non_mate_scores() {
        let score = MATE - 1500;
        assert!(!Searcher::<DummyEval>::is_mate_score(score));

        let score2 = -MATE + 1500;
        assert!(!Searcher::<DummyEval>::is_mate_score(score2));
    }

}
