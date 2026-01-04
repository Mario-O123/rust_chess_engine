//Pawn specific movegen
//here parameter square color and vector WE ABSOLUTELY NEED LAST MOVE FOR EN PASSANT
use crate::movegen::{Move, PromotionPiece};
use crate::position::{Cell, Color, Position};

pub fn gen_pawn_moves(position: &Position, moves: &mut Vec<Move>, square: usize) {
    // might want to change magic square index to more logical square identificators

    let pawn_starts_white: [usize; 8] = [31, 32, 33, 34, 35, 36, 37, 38]; //alle start index of white pawns
    let pawn_starts_black: [usize; 8] = [381, 82, 83, 84, 85, 86, 87, 88]; //all start index of black pawns 
    let pawn_promotion_rank_white: [usize; 8] = [91, 92, 93, 94, 95, 96, 97, 98]; //all index of squares where white pawn can promote 
    let pawn_promotion_rank_black: [usize; 8] = [21, 22, 23, 24, 25, 26, 27, 28]; // same but with black pawns
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
                if position.board[(square as i32 + 10) as usize] == Cell::Empty {
                    //push 1 up move to vector

                    if pawn_promotion_rank_white.contains(&((square as i32 + 10) as usize)) {
                        gen_all_promotion_pieces(square, moves, 10)
                    } else {
                        moves.push(Move::new(square, (square as i32 + 10) as usize));
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
}

//
pub fn en_passant_moves(position: &Position, moves: &mut Vec<Move>, square: usize) {
    if position.en_passant_square.is_some() {
        if let Some(en_passant_to) = position.en_passant_square {
            match position.player_to_move {
                Color::White => {
                    if (square as i32 - 9) as usize == en_passant_to
                        || (square as i32 - 11) as usize == en_passant_to
                    {
                        moves.push(Move::new_en_passant(square as usize, en_passant_to))
                    }
                }
                Color::Black => {
                    if (square as i32 + 9) as usize == en_passant_to
                        || (square as i32 + 11) as usize == en_passant_to
                    {
                        moves.push(Move::new_en_passant(square as usize, en_passant_to))
                    }
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
