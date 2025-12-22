//Pawn specific movegen
//hier parameter square color und vector AUßERDEM BRAUCHT MAN FÜR EN PASSANT LAST MOVE

//man kann hier noch zeilen reduzieren wenn man bei promotion eine funktion aufruft die jedes mögliche promotion_piece einfach als move pusht 
//anstatt in jedem block die 4 moves einzeln zu pushen


    fn gen_pawn_moves(position: &Position, moves: &mut Vec<Move>, square: usize,  last_move: Move) {
        let pawn_starts_white: [usize;8] = []; //alle indexe des startrangs weißer pawns
        let pawn_starts_black: [usize;8] = []; //alle indexe des startrangs scharzer pawns
        let pawn_promotion_rank_white: [usize;8] = [];
        let pawn_promotion_rank_black: [usize;8] = [];
        if let Cell::Piece(piece) = position.board[square] {
        match piece.color {
        Color::White => {
        if pawn_starts_white.contains(square) && position.board[(square as i8 +10) as usize]== Cell::Empty && 
        position.board[(square as i8 +20) as usize ]== Cell::Empty{
            //push 2 up move to vector
            moves.push(Move::new(square, (square+20), promotion_none , double_move_pawn_flag));
        }
        if position.board[(square as i8 +10) as usize] == Cell::Empty {
            //push 1 up move to vector
            if pawn_promotion_rank_white.contains(square+10) {
                moves.push(Move::new(square as u8, (square as i8 +10) as u8, promotion_knight , none_flag));
                moves.push(Move::new(square as u8, (square as i8 +10) as u8, promotion_bishop , none_flag));
                moves.push(Move::new(square as u8, (square as i8 +10) as u8, promotion_rook , none_flag));
                moves.push(Move::new(square as u8, (square as i8 +10) as u8, promotion_queen , none_flag));

            }else {
                moves.push(Move::new(square as u8, (square as i8 +10) as u8, promotion_none , none_flag));
            }
        }
        if let Cell::Piece(piece) = position.board[(square as i8  + 11) as usize] {
            if piece.color == Color::Black {
                // push take right to vector
                if pawn_promotion_rank_white.contains((square as i8 +11) as usize) {
                    moves.push(Move::new(square as u8, (square as i8 +11) as u8, promotion_knight , none_flag));
                    moves.push(Move::new(square as u8, (square as i8 +11) as u8, promotion_bishop , none_flag));
                    moves.push(Move::new(square as u8, (square as i8 +11) as u8, promotion_rook , none_flag));
                    moves.push(Move::new(square as u8, (square as i8 +11) as u8, promotion_queen , none_flag));

                }else {
                    moves.push(Move::new(square as u8 , (square as i8 +11) as u8, promotion_none , none_flag));
                }

        }}
        if let Cell::Piece(piece) = position.board[(square as i8  + 9) as usize] {
            if piece.color == Color::Black {
                // push take left to vector
                if pawn_promotion_rank_white.contains((square as i8 +9) as usize) {
                    moves.push(Move::new(square as u8, (square as i8 +9) as u8, promotion_knight , none_flag));
                    moves.push(Move::new(square as u8, (square as i8 +9) as u8, promotion_bishop , none_flag));
                    moves.push(Move::new(square as u8, (square as i8 +9) as u8, promotion_rook , none_flag));
                    moves.push(Move::new(square as u8, (square as i8 +9) as u8, promotion_queen , none_flag));

                }else {
                    moves.push(Move::new(square as u8, (square as i8 +9) as u8, promotion_none , none_flag));
                }
        }}
        //make en passant movelogic here with last move and chek if last move 
    }

        Color::Black =>  {
        if pawn_starts_black.contains(square) && position.board[(square as i8 -10) as usize ]== Cell::Empty && 
        position.board[(square as i8 -20) as usize ]== Cell::Empty{
            //push 2 up move to vector
            moves.push(Move::new(square as u8, (square as i8 -20) as u8, promotion_none , double_move_pawn_flag));

        }
        if position.board[(square as i8 -10) as usize] == Cell::Empty {
            //push 1 up move to vector
            if pawn_promotion_rank_black.contains((square as i8 -10)as usize) {
                moves.push(Move::new(square as u8, (square as i8 -10) as u8, promotion_knight , none_flag));
                moves.push(Move::new(square as u8, (square as i8 -10) as u8, promotion_bishop, none_flag));
                moves.push(Move::new(square as u8, (square as i8 -10) as u8, promotion_rook , none_flag));
                moves.push(Move::new(square as u8, (square as i8 -10) as u8, promotion_queen , none_flag));

            }else {
                moves.push(Move::new(square, (square-10), promotion_none , none_flag));
            }
        }
        if let Cell::Piece(piece) = position.board[(square as i8 - 11) as usize] {
            if piece.color == Color::White {
                // push take right to vector
                if pawn_promotion_rank_black.contains((square as i8 -11) as usize) {
                    moves.push(Move::new(square as u8, (square as i8 -11) as u8, promotion_knight , none_flag));
                    moves.push(Move::new(square as u8, (square as i8 -11) as u8, promotion_bishop , none_flag));
                    moves.push(Move::new(square as u8, (square as i8 -11) as u8, promotion_rook , none_flag));
                    moves.push(Move::new(square as u8, (square as i8 -11) as u8, promotion_queen , none_flag));

                }else {
                    moves.push(Move::new(square as u8, (square as i8 -11) as u8, promotion_none , none_flag));
                }
        }}
        if let Cell::Piece(piece) = position.board[(square as i8  - 9) as usize] {
            if piece.color == Color::White {
                // push take left to vector
                if pawn_promotion_rank_black.contains(square-9) {
                    moves.push(Move::new(square as u8, (square as i8 -9) as u8, promotion_knight , none_flag));
                    moves.push(Move::new(square as u8, (square as i8 -9) as u8, promotion_bishop , none_flag));
                    moves.push(Move::new(square as u8, (square as i8 -9) as u8, promotion_rook , none_flag));
                    moves.push(Move::new(square as u8, (square as i8 -9) as u8, promotion_queen , none_flag));

                }else {
                    moves.push(Move::new(square as u8, (square as i8 -9) as u8, promotion_none , none_flag));
                }
        }}
            
        }
    }
}
    }
    //2moveflag muss noch ein int sein der in flags für 2move steht
    fn en_passant_moves(position: &Position, moves: &mut Vec<Move>, square: usize, last_move: Move) {
        if let Cell::Piece(piece) = position.board[square] {
        if let Cell::Piece(last_move_piece) = position.board[last_move.to] {
        match piece.color {
        Color::White => {

        if last_move_piece.kind == PieceKind::Pawn && 
            last_move.flags == double_move_pawn_flag && 
            last_move.to == (square as i8 +1) as u8  {
                moves.push(Move::new(square as u8, (square as i8 +11) as u8, promotion_none, en_passant_flag))

            }
        else if last_move_piece.kind == PieceKind::Pawn && 
            last_move.flags == double_move_pawn_flag && 
            last_move.to == (square as i8 -1) as u8  {
                moves.push(Move::new(square as u8 , (square as i8 +9) as u8, promotion_none, en_passant_flag))
            }
        }
        Color::Black => {
            if last_move_piece.kind == PieceKind::Pawn && 
            last_move.flags == double_move_pawn_flag && 
            last_move.to == (square as i8 +1) as u8  {
                moves.push(Move::new(square as u8, (square as i8 -9) as u8 , promotion_none, en_passant_flag))

            }
        else if last_move_piece.kind == PieceKind::Pawn && 
            last_move.flags == double_move_pawn_flag && 
            last_move.to == (square as i8 -1) as u8  {
                moves.push(Move::new(square as u8, (square as i8 -11) as u8, promotion_none, en_passant_flag))
            }

        }
    }
}
        }
    }