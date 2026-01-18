//move ordering for better alpha beta pruning

use crate::movegen::Move;
use crate::position::{Position, Cell};
use crate::position::position::PieceKind;

// MVV-LVA: Most Valuable Victim - Least Valuable Attacker
const MVV_LVA: [[i32; 6]; 6] = [
    // Victim:  P    N    B    R    Q    K
    [105, 205, 305, 405, 505, 605], // Pawn attacker
    [104, 204, 304, 404, 504, 604], // Knight
    [103, 203, 303, 403, 503, 603], // Bishop
    [102, 202, 302, 402, 502, 602], // Rook
    [101, 201, 301, 401, 501, 601], // Queen
    [100, 200, 300, 400, 500, 600], // King
];

/// Order moves for better search efficiency
pub fn order_moves(position: &Position, moves: &mut [Move]) {
    moves.sort_by_cached_key(|mv| -score_move(position, mv));
}

fn score_move(position: &Position, mv: &Move) -> i32 {
    let mut score = 0;
    
    // 1. Promotions (very good)
    if mv.is_promotion() {
        score += 9000;
        if let Some(promo) = mv.promotion_piece() {
            use crate::movegen::PromotionPiece;
            score += match promo {
                PromotionPiece::Queen => 900,
                PromotionPiece::Rook => 500,
                PromotionPiece::Bishop => 300,
                PromotionPiece::Knight => 300,
            };
        }
    }
    
    // 2. Captures (MVV-LVA)
    if let Cell::Piece(victim) = position.board[mv.to_sq()] {
        if let Cell::Piece(attacker) = position.board[mv.from_sq()] {
            score += MVV_LVA[attacker.kind.idx()][victim.kind.idx()] * 100;
        }
    }
    
    // 3. Castling (slightly good)
    if mv.is_castling() {
        score += 50;
    }
    
    score
}

