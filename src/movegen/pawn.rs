//Pawn specific movegen
//here parameter square color and vector WE ABSOLUTELY NEED LAST MOVE FOR EN PASSANT


    fn gen_pawn_moves(position: &Position, moves: &mut Vec<Move>, square: usize,  last_move: Move) {
        let pawn_starts_white: [usize;8] = [81,82,83,84,85,86,87,88]; //all white index
        let pawn_starts_black: [usize;8] = [31,32,33,34,35,36,37,38]; //all Black index
        let pawn_promotion_rank_white: [usize;8] = [21,22,23,24,25,26,27,28];
        let pawn_promotion_rank_black: [usize;8] = [91,92,93,94,95,96,97,98];
        if let Cell::Piece(piece) = position.board[square] {
        match piece.color {
        Color::White => {
        if pawn_starts_white.contains(&square) && position.board[(square as i8 +10) as usize]== Cell::Empty && 
        position.board[(square as i8 +20) as usize ]== Cell::Empty{
            //push 2 up move to vector
            moves.push(Move::new(square, (square+20)));
        }
        if position.board[(square as i8 +10) as usize] == Cell::Empty {
            //push 1 up move to vector
            if pawn_promotion_rank_white.contains(&square+10) {
                moves.push(Move::new_promotion(square as u8, (square as i8 +10) as u8, PromotionPiece::Knight));
                moves.push(Move::new_promotion(square as u8, (square as i8 +10) as u8, PromotionPiece::Bishop));
                moves.push(Move::new_promotion(square as u8, (square as i8 +10) as u8, PromotionPiece::Rook));
                moves.push(Move::new_promotion(square as u8, (square as i8 +10) as u8, PromotionPiece::Queen));

            }else {
                moves.push(Move::new(square as u8, (square as i8 +10) as u8));
            }
        }
        if let Cell::Piece(piece) = position.board[(square as i8  + 11) as usize] {
            if piece.color == Color::Black {
                // push take right to vector
                if pawn_promotion_rank_white.contains((&square as i8 +11) as usize) {
                moves.push(Move::new_promotion(square as u8, (square as i8 +11) as u8, PromotionPiece::Knight));
                moves.push(Move::new_promotion(square as u8, (square as i8 +11) as u8, PromotionPiece::Bishop));
                moves.push(Move::new_promotion(square as u8, (square as i8 +11) as u8, PromotionPiece::Rook));
                moves.push(Move::new_promotion(square as u8, (square as i8 +11) as u8, PromotionPiece::Queen));

                }else {
                    moves.push(Move::new(square as u8 , (square as i8 +11) as u8));
                }

        }}
        if let Cell::Piece(piece) = position.board[(square as i8  + 9) as usize] {
            if piece.color == Color::Black {
                // push take left to vector
                if pawn_promotion_rank_white.contains((&square as i8 +9) as usize) {
                moves.push(Move::new_promotion(square as u8, (square as i8 +9) as u8, PromotionPiece::Knight));
                moves.push(Move::new_promotion(square as u8, (square as i8 +9) as u8, PromotionPiece::Bishop));
                moves.push(Move::new_promotion(square as u8, (square as i8 +9) as u8, PromotionPiece::Rook));
                moves.push(Move::new_promotion(square as u8, (square as i8 +9) as u8, PromotionPiece::Queen));

                }else {
                    moves.push(Move::new(square as u8, (square as i8 +9) as u8));
                }
        }}
        //make en passant movelogic here with last move and chek if last move 
    }

        Color::Black =>  {
        if pawn_starts_black.contains(&square) && position.board[(square as i8 -10) as usize ]== Cell::Empty && 
        position.board[(square as i8 -20) as usize ]== Cell::Empty{
            //push 2 up move to vector
            moves.push(Move::new(square as u8, (square as i8 -20) as u8));

        }
        if position.board[(square as i8 -10) as usize] == Cell::Empty {
            //push 1 up move to vector
            if pawn_promotion_rank_black.contains((&square as i8 -10)as usize) {
                moves.push(Move::new_promotion(square as u8, (square as i8 -10) as u8, PromotionPiece::Knight));
                moves.push(Move::new_promotion(square as u8, (square as i8 -10) as u8, PromotionPiece::Bishop));
                moves.push(Move::new_promotion(square as u8, (square as i8 -10) as u8, PromotionPiece::Rook));
                moves.push(Move::new_promotion(square as u8, (square as i8 -10) as u8, PromotionPiece::Queen));

            }else {
                moves.push(Move::new(square, (square-10)));
            }
        }
        if let Cell::Piece(piece) = position.board[(square as i8 - 11) as usize] {
            if piece.color == Color::White {
                // push take right to vector
                if pawn_promotion_rank_black.contains((&square as i8 -11) as usize) {
                moves.push(Move::new_promotion(square as u8, (square as i8 -11) as u8, PromotionPiece::Knight));
                moves.push(Move::new_promotion(square as u8, (square as i8 -11) as u8, PromotionPiece::Bishop));
                moves.push(Move::new_promotion(square as u8, (square as i8 -11) as u8, PromotionPiece::Rook));
                moves.push(Move::new_promotion(square as u8, (square as i8 -11) as u8, PromotionPiece::Queen));

                }else {
                    moves.push(Move::new(square as u8, (square as i8 -11) as u8));
                }
        }}
        if let Cell::Piece(piece) = position.board[(square as i8  - 9) as usize] {
            if piece.color == Color::White {
                // push take left to vector
                if pawn_promotion_rank_black.contains(&square-9) {
                moves.push(Move::new_promotion(square as u8, (square as i8 -9) as u8, PromotionPiece::Knight));
                moves.push(Move::new_promotion(square as u8, (square as i8 -9) as u8, PromotionPiece::Bishop));
                moves.push(Move::new_promotion(square as u8, (square as i8 -9) as u8, PromotionPiece::Rook));
                moves.push(Move::new_promotion(square as u8, (square as i8 -9) as u8, PromotionPiece::Queen));

                }else {
                    moves.push(Move::new(square as u8, (square as i8 -9) as u8));
                }
        }}
            
        }
    }
}
    }
    fn en_passant_moves(position: &Position, moves: &mut Vec<Move>, square: usize, last_move: Move) {
        if last_move.is_null() {
            return;
        }
        if let Cell::Piece(piece) = position.board[square] {
        if let Cell::Piece(last_move_piece) = position.board[last_move.to] {
        match piece.color {
        Color::White => {

        if last_move_piece.kind == PieceKind::Pawn && 
            last_move.to == (last_move.from -20) && 
            last_move.to == (square as i8 +1) as u8  {
                moves.push(Move::new_en_passant(square as u8, (square as i8 +11) as u8));

            }
        else if last_move_piece.kind == PieceKind::Pawn && 
            last_move.to == (last_move.from -20) && 
            last_move.to == (square as i8 -1) as u8  {
                moves.push(Move::new_en_passant(square as u8, (square as i8 +9) as u8));
            }
        }
        Color::Black => {
            if last_move_piece.kind == PieceKind::Pawn && 
            last_move.to == (last_move.from +20) && 
            last_move.to == (square as i8 +1) as u8  {
                moves.push(Move::new_en_passant(square as u8, (square as i8 -11) as u8));

            }
        else if last_move_piece.kind == PieceKind::Pawn && 
            last_move.to == (last_move.from +20) && 
            last_move.to == (square as i8 -1) as u8  {
                moves.push(Move::new_en_passant(square as u8, (square as i8 -9) as u8));
            }

        }
    }
}
        }
    }