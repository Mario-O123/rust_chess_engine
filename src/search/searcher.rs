use std::time::Instant;

use crate::evaluation::Evaluator;
use crate::movegen::{
    generate_legal_captures_in_place, 
    generate_legal_moves_in_place,
    is_in_check,
    Move,};
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

impl <E: Evaluator> Searcher<E> {
    pub fn new(eval: E) -> Self {
        Self { eval, 
            nodes: 0, 
            start: Instant::now(), 
            limits: SearchLimits { max_depth: 1, max_nodes: None, max_time_ms: None }, 
            history: Vec::new(),
            move_buf: Vec::new() }
    }

    pub fn search(&mut self, pos: &mut Position, limits: SearchLimits) -> SearchResult {
        self.limits = limits;
        self.nodes = 0;
        self.start = Instant::now();

        self.history.clear();
        self.history.push(pos.zobrist);

        let mut best_move = Move::NULL;
        let mut best_score = -INF;
        let mut reached_depth = 0;

        for d in 1..=self.limits.max_depth {
            let (mv, sc) = self.root(pos, d as i32);
            if mv.is_null() {
                break;
            }

            best_move = mv;
            best_score = sc;
            reached_depth = d;

            if self.should_stop() {
                break;
            }
        }
        SearchResult { 
            best_move, 
            score_cp: best_score, 
            depth: reached_depth, 
            nodes: self.nodes }

    }

    fn root (&mut self, pos: &mut Position, depth: i32) -> (Move, i32) {
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

        let mut scored_moves: Vec<(Move, i32)> = self.move_buf.iter()
            .map(|&m| (m, Self::move_order_score(pos, m))).collect();

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

        let mut scored_moves: Vec<(Move, i32)> = self.move_buf.iter()
            .map(|&m| (m, Self::move_order_score(pos, m))).collect();

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

    fn quiescence(
        &mut self,
        pos: &mut Position,
        ply: i32,
        mut alpha: i32,
        beta: i32,
    ) -> i32 {
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

            let mut scored_moves: Vec<(Move, i32)> = self.move_buf.iter()
            .map(|&m| (m, Self::move_order_score(pos, m))).collect();

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
        let mut scored_moves: Vec<(Move, i32)> = self.move_buf.iter()
            .map(|&m| (m, Self::move_order_score(pos, m))).collect();

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
    use crate::position::Position;
    use crate::evaluation::classical::ClassicalEval;

    // Test 1: Grundlegende Funktionalität - Findet einen legalen Zug
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

    // Test 2: Matt in 1 erkennen (Scholar's Mate Setup)
    #[test]
    fn test_finds_mate_in_one() {
        // Position: Weiß am Zug kann mit Qf7# Matt setzen
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
        
        // Score sollte sehr negativ sein (Matt gegen Schwarz)
        assert!(result.score_cp < -25000);
    }

    // Test 3: Stalemate erkennen
    #[test]
    fn test_recognizes_stalemate() {
        // Klassisches Stalemate: König in Ecke, Dame blockiert alles
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
        
        assert!(result.best_move.is_null()); // Keine legalen Züge
        assert_eq!(result.score_cp, 0); // Stalemate = Draw
    }

    // Test 4: 50-Move Rule
    #[test]
    fn test_fifty_move_rule() {
        let mut pos = Position::starting_position();
        pos.half_move_clock = 100; // 50-move rule erreicht
        
        let eval = ClassicalEval::new();
        let mut searcher = Searcher::new(eval);
        
        let limits = SearchLimits {
            max_depth: 3,
            max_nodes: None,
            max_time_ms: None,
        };
        
        let result = searcher.search(&mut pos, limits);
        
        // Sollte 0 (Draw) returnen wegen 50-move rule
        assert_eq!(result.score_cp, 0);
    }

    // Test 5: Repetition Detection
    #[test]
    fn test_repetition_detection() {
        let mut pos = Position::starting_position();
        let eval = ClassicalEval::new();
        let mut searcher = Searcher::new(eval);
        
        // Simuliere Repetition durch manuelle History
        searcher.history.push(pos.zobrist);
        searcher.history.push(12345);
        searcher.history.push(pos.zobrist); // Repetition!
        
        assert!(searcher.is_repetition(pos.zobrist));
    }

    // Test 6: Time Limit respektieren
    #[test]
    fn test_respects_time_limit() {
        let mut pos = Position::starting_position();
        let eval = ClassicalEval::new();
        let mut searcher = Searcher::new(eval);
        
        let limits = SearchLimits {
            max_depth: 100, // Sehr tief
            max_nodes: None,
            max_time_ms: Some(100), // Nur 100ms
        };
        
        let start = std::time::Instant::now();
        let result = searcher.search(&mut pos, limits);
        let elapsed = start.elapsed().as_millis();
        
        // Sollte innerhalb ~100-200ms stoppen
        assert!(elapsed < 300);
        assert!(result.depth < 100); // Hat nicht volle Tiefe erreicht
    }

    // Test 7: Node Limit respektieren
    #[test]
    fn test_respects_node_limit() {
        let mut pos = Position::starting_position();
        let eval = ClassicalEval::new();
        let mut searcher = Searcher::new(eval);
        
        let limits = SearchLimits {
            max_depth: 100,
            max_nodes: Some(1000), // Nur 1000 Nodes
            max_time_ms: None,
        };
        
        let result = searcher.search(&mut pos, limits);
        
        assert!(result.nodes <= 1100); // Etwas Toleranz für Überlauf
        assert!(result.depth < 100);
    }

    // Test 8: Captures werden höher bewertet als Quiet Moves
    #[test]
    fn test_move_ordering_prefers_captures() {
        let mut pos = Position::starting_position();
        let eval = ClassicalEval::new();
        let searcher = Searcher::new(eval);
        
        // Setup: Eine Position mit Capture und Quiet Move
        let fen = "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 2";
        let pos = Position::from_fen(fen).unwrap();
        
        // Normale Züge
        let quiet_move = Move::new(52, 62); // d2-d3
        let capture_move = Move::new(54, 64); // e4xe5
        
        let quiet_score = Searcher::<ClassicalEval>::move_order_score(&pos, quiet_move);
        let capture_score = Searcher::<ClassicalEval>::move_order_score(&pos, capture_move);
        
        assert!(capture_score > quiet_score);
    }

    // Test 9: Promotions werden am höchsten bewertet
    #[test]
    fn test_move_ordering_prefers_promotions() {
        let eval = ClassicalEval::new();
        let searcher = Searcher::new(eval);
        
        let fen = "4k3/P7/8/8/8/8/8/4K3 w - - 0 1";
        let pos = Position::from_fen(fen).unwrap();
        
        use crate::movegen::PromotionPiece;
        let promotion = Move::new_promotion(81, 91, PromotionPiece::Queen);
        let normal = Move::new(25, 35); // e1-e2
        
        let promo_score = Searcher::<ClassicalEval>::move_order_score(&pos, promotion);
        let normal_score = Searcher::<ClassicalEval>::move_order_score(&pos, normal);
        
        assert!(promo_score > 90000);
        assert!(promo_score > normal_score);
    }

    // Test 10: Reached Depth ist korrekt bei frühem Stop
    #[test]
    fn test_reached_depth_correct_on_early_stop() {
        let mut pos = Position::starting_position();
        let eval = ClassicalEval::new();
        let mut searcher = Searcher::new(eval);
        
        let limits = SearchLimits {
            max_depth: 10,
            max_nodes: Some(100), // Sehr wenige Nodes
            max_time_ms: None,
        };
        
        let result = searcher.search(&mut pos, limits);
        
        // reached_depth sollte kleiner sein als max_depth
        assert!(result.depth < 10);
        // Nodes sollte ungefähr beim Limit sein
        assert!(result.nodes <= 150);
    }

    // BONUS Test 11: Quiescence findet taktische Züge
    #[test]
    fn test_quiescence_finds_captures() {
        let mut pos = Position::starting_position();
        let eval = ClassicalEval::new();
        let mut searcher = Searcher::new(eval);
        
        // Position mit hängender Dame
        let fen = "rnbqkb1r/pppp1ppp/5n2/4p3/3P4/5N2/PPP1QPPP/RNB1KB1R b KQkq - 0 1";
        let mut pos = Position::from_fen(fen).unwrap();
        
        let limits = SearchLimits {
            max_depth: 4,
            max_nodes: None,
            max_time_ms: None,
        };
        
        let result = searcher.search(&mut pos, limits);
        
        // Schwarz sollte Qxe2 oder ähnlich finden
        // Score sollte deutlich positiv für Schwarz sein
        assert!(result.score_cp < -300); // Mindestens Figurgewinn
    }

    // BONUS Test 12: Position bleibt konsistent nach Search
    #[test]
    fn test_position_unchanged_after_search() {
        let mut pos = Position::starting_position();
        let original_zobrist = pos.zobrist;
        let original_player = pos.player_to_move;
        
        let eval = ClassicalEval::new();
        let mut searcher = Searcher::new(eval);
        
        let limits = SearchLimits {
            max_depth: 5,
            max_nodes: None,
            max_time_ms: None,
        };
        
        searcher.search(&mut pos, limits);
        
        // Position sollte unverändert sein
        assert_eq!(pos.zobrist, original_zobrist);
        assert_eq!(pos.player_to_move, original_player);
    }
}

