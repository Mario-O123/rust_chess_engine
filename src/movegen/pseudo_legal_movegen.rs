


// const sliding: [bool;5] = [false, true, true, true, false]; //knight bishop rook queen king 
//braucht man denke nicht wenn man jedes piece sowieso seperat behandelt aber vllt nicht jedes piece seperat behandeln sondern
// nur in sliding und nicht sliding unterscheiden /  und halt in pawn moves am anfang sowieso 
const knight_offsets:[i8; 8] = [21, (-21), 19 , (-19), 12 , (-12) , 8 , (-8) ];
const bishop_offsets:[i8; 4] = [11, (-11), 9, (-9)];
const rook_offsets:[i8; 4] = [1, (-1), 10, (-10)];
const king_queen_offsets:[i8; 8] = [1 , (-1), 10, (-10), 11 , (-11), 9, (-9)];

fn movegen(position : &Position) -> Vec<Move> {
    let mut move_list = Vec::new();
    for (index, piece) in board.iter().enumerate() { 
    if let Cell::Piece(piece) = &position.board[index] {
        if piece.color == player_to_move {
    
        match piece.kind {
            PieceKind::Knight => { 
                gen_jumping_moves();
            }PieceKind::Bishop => {
                gen_sliding_moves();
            }PieceKind::Pawn => {
                gen_pawn_moves();
                en_passant_moves();
            }PieceKind::Rook => {
                gen_sliding_moves();
            }PieceKind::Queen => {
                gen_sliding_moves();
            }PieceKind::King => {
                gen_jumping_moves();
                gen_castling_moves();
            }
        }}
}
    }
return move_list;
}