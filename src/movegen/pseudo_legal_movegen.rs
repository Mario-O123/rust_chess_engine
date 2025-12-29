


// const sliding: [bool;5] = [false, true, true, true, false]; //knight bishop rook queen king 
//braucht man denke nicht wenn man jedes piece sowieso seperat behandelt aber vllt nicht jedes piece seperat behandeln sondern
// nur in sliding und nicht sliding unterscheiden /  und halt in pawn moves am anfang sowieso 
const knight_offsets:[i8; 8] = [21, (-21), 19 , (-19), 12 , (-12) , 8 , (-8) ];
const bishop_offsets:[i8; 4] = [11, (-11), 9, (-9)];
const rook_offsets:[i8; 4] = [1, (-1), 10, (-10)];
const king_queen_offsets:[i8; 8] = [1 , (-1), 10, (-10), 11 , (-11), 9, (-9)];

fn generate_pseudo_legal_moves(position : &Position, last_move: Move) -> Vec<Move> {
    let mut move_list = Vec::new();
    for square120 in 21..=98 {
        if !is_on_board(square120) {
            continue;
        } 
    if let Cell::Piece(piece) = position.board[square120] {
        if piece.color == player_to_move {
    
        match piece.kind {
            PieceKind::Knight => { 
                gen_jumping_moves(position, &mut move_list, square120, &knight_offsets);
            }PieceKind::Bishop => {
                gen_sliding_moves(position, &mut move_list, square120, &bishop_offsets);
            }PieceKind::Pawn => {
                gen_pawn_moves(position, &mut move_list, square120, position.last_move);
                en_passant_moves(position, &mut move_list, square120, position.last_move);
            }PieceKind::Rook => {
                gen_sliding_moves(position, &mut move_list, square120, &rook_offsets);
            }PieceKind::Queen => {
                gen_sliding_moves(position, &mut move_list, square120, &king_queen_offsets);
            }PieceKind::King => {
                gen_jumping_moves(position, &mut move_list, square120, &king_queen_offsets);
                gen_castling_moves(position, &mut move_list, square120);
            }
        }}
}
    }
return move_list;
}