//Pawn specific movegen
//here parameter square color and vector WE ABSOLUTELY NEED LAST MOVE FOR EN PASSANT
use crate::movegen::{Move, PromotionPiece};
use crate::position::{Cell, Color, Position};

const PAWN_START_WHITE: [usize; 8] = [81, 82, 83, 84, 85, 86, 87, 88];
const PAWN_START_BLACK: [usize; 8] = [31, 32, 33, 34, 35, 36, 37, 38];
const PROMOTION_RANK_WHITE: [usize; 8] = [21, 22, 23, 24, 25, 26, 27, 28];
const PROMOTION_RANK_BLACK: [usize; 8] = [91, 92, 93, 94, 95, 96, 97, 98];


pub fn gen_pawn_moves(position: &Position, moves: &mut Vec<Move>, square: usize) {
    let Cell::Piece(piece) = position.board[square] else {
        return;
    };

    match piece.color {
        Color::White => {
            // Double push from starting position
            if PAWN_START_WHITE.contains(&square) {
                let one_forward = square + 10;
                let two_forward = square + 20;
                
                if two_forward < 120
                    && position.board[one_forward] == Cell::Empty
                    && position.board[two_forward] == Cell::Empty
                {
                    //uses now new_pawn_double for En-Passant-Square
                    moves.push(Move::new_pawn_double(square, two_forward));
                }
            }

            // Single push forward
            let forward = square + 10;
            if forward < 120 && position.board[forward] == Cell::Empty {
                if PROMOTION_RANK_WHITE.contains(&forward) {
                    gen_all_promotion_pieces(square, moves, 10);
                } else {
                    moves.push(Move::new(square, forward));
                }
            }

            // Capture right (diagonal +11)
            let capture_right = square as i32 + 11;
            if capture_right >= 0 && capture_right < 120 {
                let capture_right = capture_right as usize;
                //Offboard-Check added
                if position.board[capture_right] != Cell::Offboard {
                    if let Cell::Piece(target) = position.board[capture_right] {
                        if target.color == Color::Black {
                            if PROMOTION_RANK_WHITE.contains(&capture_right) {
                                gen_all_promotion_pieces(square, moves, 11);
                            } else {
                                moves.push(Move::new(square, capture_right));
                            }
                        }
                    }
                }
            }

            // Capture left (diagonal +9)
            let capture_left = square as i32 + 9;
            if capture_left >= 0 && capture_left < 120 {
                let capture_left = capture_left as usize;
                //Offboard-Check added
                if position.board[capture_left] != Cell::Offboard {
                    if let Cell::Piece(target) = position.board[capture_left] {
                        if target.color == Color::Black {
                            if PROMOTION_RANK_WHITE.contains(&capture_left) {
                                gen_all_promotion_pieces(square, moves, 9);
                            } else {
                                moves.push(Move::new(square, capture_left));
                            }
                        }
                    }
                }
            }

            // En-Passant moves added
            en_passant_moves(position, moves, square);
        }

        Color::Black => {
            // Double push from starting position
            if PAWN_START_BLACK.contains(&square) {
                let one_forward = square as i32 - 10;
                let two_forward = square as i32 - 20;
                
                if one_forward >= 0 && two_forward >= 0 {
                    let one_forward = one_forward as usize;
                    let two_forward = two_forward as usize;
                    
                    if position.board[one_forward] == Cell::Empty
                        && position.board[two_forward] == Cell::Empty
                    {
                        //uses new_pawn_double
                        moves.push(Move::new_pawn_double(square, two_forward));
                    }
                }
            }

            // Single push forward
            let forward = square as i32 - 10;
            if forward >= 0 && forward < 120 {
                let forward = forward as usize;
                if position.board[forward] == Cell::Empty {
                    if PROMOTION_RANK_BLACK.contains(&forward) {
                        gen_all_promotion_pieces(square, moves, -10);
                    } else {
                        moves.push(Move::new(square, forward));
                    }
                }
            }

            // Capture right (diagonal -11)
            let capture_right = square as i32 - 11;
            if capture_right >= 0 && capture_right < 120 {
                let capture_right = capture_right as usize;
                //Offboard-Check added
                if position.board[capture_right] != Cell::Offboard {
                    if let Cell::Piece(target) = position.board[capture_right] {
                        if target.color == Color::White {
                            if PROMOTION_RANK_BLACK.contains(&capture_right) {
                                gen_all_promotion_pieces(square, moves, -11);
                            } else {
                                moves.push(Move::new(square, capture_right));
                            }
                        }
                    }
                }
            }

            // Capture left (diagonal -9)
            let capture_left = square as i32 - 9;
            if capture_left >= 0 && capture_left < 120 {
                let capture_left = capture_left as usize;
                // Offboard-Check added
                if position.board[capture_left] != Cell::Offboard {
                    if let Cell::Piece(target) = position.board[capture_left] {
                        if target.color == Color::White {
                            if PROMOTION_RANK_BLACK.contains(&capture_left) {
                                gen_all_promotion_pieces(square, moves, -9);
                            } else {
                                moves.push(Move::new(square, capture_left));
                            }
                        }
                    }
                }
            }

            //En-Passant Züge added
            en_passant_moves(position, moves, square);
        }
    }
}


pub fn en_passant_moves(
    position: &Position,
    moves: &mut Vec<Move>,
    square: usize, /* , last_move: Move */
) {
    if let Some(en_passant_to) = position.en_passant_square {
        let en_passant_idx = en_passant_to.as_usize();

        match position.player_to_move {
            Color::White => {
                if (square + 9 < 120 && square + 9 == en_passant_idx)
                    || (square + 11 < 120 && square + 11 == en_passant_idx)
                {
                    moves.push(Move::new_en_passant(square, en_passant_idx));
                }
            }
            Color::Black => {
                if (square >= 9 && square - 9 == en_passant_idx)
                    || (square >= 11 && square - 11 == en_passant_idx)
                {
                    moves.push(Move::new_en_passant(square, en_passant_idx));
                }
            }
        }
    }

    /* *
            if last_move.is_null() {
                return;
            }
            if let Cell::Piece(piece) = position.board[square] {
            if let Cell::Piece(last_move_piece) = position.board[last_move.to] {
            match piece.color {
            Color::White => {

            if last_move_piece.kind == PieceKind::Pawn &&
                last_move.to == (last_move.from -20) &&
                last_move.to == (square as i32 +1) as usize  {
                    moves.push(Move::new_en_passant(square as usize, (square as i32 +11) as usize));

                }
            else if last_move_piece.kind == PieceKind::Pawn &&
                last_move.to == (last_move.from -20) &&
                last_move.to == (square as i32 -1) as usize  {
                    moves.push(Move::new_en_passant(square as usize, (square as i32 +9) as usize));
                }
            }
            Color::Black => {
                if last_move_piece.kind == PieceKind::Pawn &&
                last_move.to == (last_move.from +20) &&
                last_move.to == (square as i32 +1) as usize  {
                    moves.push(Move::new_en_passant(square as usize, (square as i32 -11) as usize));

                }
            else if last_move_piece.kind == PieceKind::Pawn &&
                last_move.to == (last_move.from +20) &&
                last_move.to == (square as i32 -1) as usize  {
                    moves.push(Move::new_en_passant(square as usize, (square as i32 -9) as usize));
                }

    } }}}*/
}

fn gen_all_promotion_pieces(square: usize, moves: &mut Vec<Move>, offset: i32) {
    moves.push(Move::new_promotion(
        square,
        (square as i32 + offset) as usize,
        PromotionPiece::Knight,
    ));
    moves.push(Move::new_promotion(
        square,
        (square as i32 + offset) as usize,
        PromotionPiece::Bishop,
    ));
    moves.push(Move::new_promotion(
        square,
        (square as i32 + offset) as usize,
        PromotionPiece::Rook,
    ));
    moves.push(Move::new_promotion(
        square,
        (square as i32 + offset) as usize,
        PromotionPiece::Queen,
    ));
}

/* pub fn gen_pawn_moves(position: &Position, moves: &mut Vec<Move>, square: usize) {
    // might want to change magic square index to more logical square identificators

    let pawn_starts_white: [usize; 8] = [81, 82, 83, 84, 85, 86, 87, 88]; //alle start index of white pawns
    let pawn_starts_black: [usize; 8] = [31, 32, 33, 34, 35, 36, 37, 38]; //all start index of black pawns
    let pawn_promotion_rank_white: [usize; 8] = [21, 22, 23, 24, 25, 26, 27, 28]; //all index of squares where white pawn can promote
    let pawn_promotion_rank_black: [usize; 8] = [91, 92, 93, 94, 95, 96, 97, 98]; // same but with black pawns
    if let Cell::Piece(piece) = position.board[square] {
        match piece.color {
            Color::White => {
                if pawn_starts_white.contains(&square)
                    && position.board[(square as i32 + 10) as usize] == Cell::Empty
                    && position.board[(square as i32 + 20) as usize] == Cell::Empty
                {
                    //push 2 up move to vector
                    moves.push(Move::new(square, square + 20));
                }

                let target = square as i32 + 10;
                if target >= 0 && target < 120 {
                    if position.board[target as usize] == Cell::Empty {
                    //push 1 up move to vector

                    if pawn_promotion_rank_white.contains(&(target as usize)) {
                        gen_all_promotion_pieces(square, moves, 10)
                    } else {
                        moves.push(Move::new(square, target as usize));
                    }
                }
                if let Cell::Piece(piece) = position.board[(square as i32 + 11) as usize] {
                    if piece.color == Color::Black {
                        // push take right to vector

                        if pawn_promotion_rank_white.contains(&((square as i32 + 11) as usize)) {
                            gen_all_promotion_pieces(square, moves, 11)
                        } else {
                            moves.push(Move::new(square as usize, (square as i32 + 11) as usize));
                        }
                    }
                }
                if let Cell::Piece(piece) = position.board[(square as i32 + 9) as usize] {
                    if piece.color == Color::Black {
                        // push take left to vector

                        if pawn_promotion_rank_white.contains(&((square as i32 + 9) as usize)) {
                            gen_all_promotion_pieces(square, moves, 9)
                        } else {
                            moves.push(Move::new(square as usize, (square as i32 + 9) as usize));
                        }
                    }
                }
                }
                
                //make en passant movelogic here with last move and chek if last move
            }

            Color::Black => {
                if pawn_starts_black.contains(&square)
                    && position.board[(square as i32 - 10) as usize] == Cell::Empty
                    && position.board[(square as i32 - 20) as usize] == Cell::Empty
                {
                    //push 2 up move to vector
                    moves.push(Move::new(square as usize, (square as i32 - 20) as usize));
                }
                if position.board[(square as i32 - 10) as usize] == Cell::Empty {
                    //push 1 up move to vector

                    if pawn_promotion_rank_black.contains(&((square as i32 - 10) as usize)) {
                        gen_all_promotion_pieces(square, moves, -10)
                    } else {
                        moves.push(Move::new(square, square - 10));
                    }
                }
                if let Cell::Piece(piece) = position.board[(square as i32 - 11) as usize] {
                    if piece.color == Color::White {
                        // push take right to vector

                        if pawn_promotion_rank_black.contains(&((square as i32 - 11) as usize)) {
                            gen_all_promotion_pieces(square, moves, -11)
                        } else {
                            moves.push(Move::new(square as usize, (square as i32 - 11) as usize));
                        }
                    }
                }
                if let Cell::Piece(piece) = position.board[(square as i32 - 9) as usize] {
                    if piece.color == Color::White {
                        // push take left to vector

                        if pawn_promotion_rank_black.contains(&((square as i32 - 9) as usize)) {
                            gen_all_promotion_pieces(square, moves, -9)
                        } else {
                            moves.push(Move::new(square as usize, (square as i32 - 9) as usize));
                        }
                    }
                }
            }
        }
    }
} */

