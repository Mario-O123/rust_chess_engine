//Knight/Bishop/Queen/King
//hier parameter square color offsets und der vector 



fn gen_sliding_moves(position: &Position, moves: &mut Vec<Move> , square: usize , piece_offsets: &[i8] ) {
     for offset in piece_offsets {
                    target = square as i8 + offset;
                    while position.board[target as usize] != Cell::Offboard {
                        if target == Cell::Empty {
                            // push move 
                            moves.push(Move::new(square as u8 , target as u8 , promotion_none , none_flag));
                            target += offset;
                        }else if matches!((position.board[target], position.board[square]), (Cell::Piece(slider_target), Cell::Piece(slider_square)) 
                                 if slider_target.color == slider_square.color.opposite()){
                            //push move with capture
                            moves.push(Move::new(square  as u8, target as u8 , promotion_none , none_flag));

                            break;
                        }
                }
                
            }
}
//hier könnte man zeilen sparen anstatt für empty und enemy piece zu cheken einen check für nicht offboard oder ma lässt so wenn man noch ne capture flag will
fn gen_jumping_moves(position: &Position, moves: &mut Vec<Move> , square: usize , piece_offsets: &[i8] ) {
    for offset in piece_offsets {
                if position.board[(square as i8 + offset) as usize] == Cell::Empty {
                    //push move to vector
                    moves.push(Move::new(square as u8, (square as i8+ offset) as u8 , promotion_none , none_flag));

                }else if matches!((position.board[(square as i8+ offset)as usize ], position.board[square]), (Cell::Piece(jumper_target), Cell::Piece(jumper_square)) 
                         if jumper_target.color == jumper_square.color.opposite()) {
                        //push move to vector with capture flag
                        moves.push(Move::new(square as u8, (square + offset) as u8, promotion_none , none_flag));

                }
            }

}

// hier braucht man color castling rights struct aus gamestate und dann noch schauen ob die felder zwischen king und rook None sind
fn gen_castling_moves(position : &Position , moves: &mut Vec<Move> , square: usize) {
match position.board[square] {
    Cell::Piece(Piece { color : Color::White }) => {
        if position.castling_rights & 0b0010 !=0 && position.board[92]==Cell::Empty && position.board[93]==Cell::Empty && position.board[94]==Cell::Empty
        && matches!(position.board[91], Cell::Piece(Piece {kind: PieceKind::Rook, ..})) {
            moves.push(Move::new(square as u8, 93 , promotion_none , castling_flag));

        } 
        if position.castling_rights & 0b0001 !=0 && position.board[96]==Cell::Empty && position.board[97]==Cell::Empty
        && && matches!(position.board[98], Cell::Piece(Piece {kind: PieceKind::Rook, ..})){
            moves.push(Move::new(square as u8, 97 , promotion_none , castling_flag));

        }
    }
    Color::Black => {
        if position.castling_rights & 0b1000 !=0 && position.board[22]== Cell::Empty && position.board[23] == Cell::Empty && position.board[24]==Cell::Empty 
        && && matches!(position.board[21], Cell::Piece(Piece {kind: PieceKind::Rook, ..})){
            moves.push(Move::new(square as u8, 23 , promotion_none , castling_flag));

        }
        if position.castling_rights & 0b0100 !=0  && position.board[26]==Cell::Empty && position.board[27]==Cell::Empty 
        && && matches!(position.board[28], Cell::Piece(Piece {kind: PieceKind::Rook, ..})){
            moves.push(Move::new(square as u8, 27 , promotion_none , castling_flag));

        }
    }
}


}