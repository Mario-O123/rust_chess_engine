use std::time::Instant;

use crate::evaluation::Evaluator;
use crate::movegen::{
    generate_legal_captures_in_place, 
    generate_legal_moves_in_place,
    is_in_check,
    Move,};
use crate::position::{Cell, Color, PieceKind, Position};

const INF: i32 = 50000;
const MATE:i32 = 30000;

#[derive(Clone, Copy)]
pub struct SearchLimits {
    pub max_depth: u8,
    pub max_nodes: Option<u64>,
    pub max_time_ms: Option<u64>,
}

pub struct Searchresult {
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

    pub fn search(&mut self, pos: &Position, limits: SearchLimits) -> Searchresult {
        self.limits = limits;
        self.nodes = 0;
        self.start = Instant::now();

        self.history.clear();
        self.history.push(pos.zobrist);

        let mut best_move = Move::NULL;
        let mut best_score = -INF;

        for d in 1..=self.limits.max_depth {
            let (mv, sc) = self.root(pos, d as i32);
            if mv.is_null() {
                break;
            }

            best_move = mv;
            best_score = sc;

            if self.should_stop() {
                break;
            }
        }
        Searchresult { 
            best_move, 
            score_cp: best_score, 
            depth: self.limits.max_depth, 
            nodes: self.nodes }

    }

    fn root (&mut self, pos: &Position, depth: i32) -> (Move, i32) {
        generate_legal_moves_in_place(pos, &mut self.move_buf);
        if self.move_buf.is_empty() {
            return (Move::NULL, self.terminal_score(pos, 0));
        }

        //simple ordering
        self.move_buf.sort_by_key(|&m| -self.move_order_score(pos, m));

        let mut best_mv = Move::NULL;
        let mut best = -INF;
        let mut alpha = -INF;
        let mut beta = INF;

        for mv in self.move_buf.iter().copied() {
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
        pos: &Position,
        depth: i32,
        ply: i32,
        mut alpha: i32,
        beta: i32,
    ) -> i32 {
        self.nodes += 1;
        if self.should_stop() {
            return 0;
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

        generate_legal_moves_in_place(pos, &mut self.move_buf);
        if self.move_buf.is_empty() {
            return self.terminal_score(pos, ply);
        }

        self.move_buf.sort_by_key(|&m| -self.move_order_score(pos, m));

        for mv in self.move_buf.iter().copied() {
            let undo = pos.make_move_with_undo(mv);
            self.history.push(pos.zobrist);

            let score = -self.negamax(pos, depth - 1, ply + 1, -beta, -alpha);

            self.history.pop();
            pos.undo_move(mv);

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
        pos: & Position,
        ply: i32,
        mut alpha: i32,
        beta: i32,
    ) -> i32 {
        self.nodes += 1;
        if self.should_stop() {
            return 0;
        }

        //if in check also allow evasion not only captures
        let stm = pos.player_to_move;
        if is_in_check(pos, stm) {
            generate_legal_moves_in_place(pos, &mut self.move_buf);
            if self.move_buf.is_empty() {
                return -MATE + ply as i32;
            }

            self.move_buf.sort_by_key(|&m| -self.move_order_score(pos, m));

            for mv in self.move_buf.iter().copied() {
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

        generate_legal_captures_in_place(pos, &mut self.move_buf);
        self.move_buf.sort_by_key(|&m| -self.move_order_score(pos, m));

        for mv in self.move_buf.iter().copied() {
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
    fn move_order_score(&self, pos: &Position, mv: Move) -> i32 {
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

