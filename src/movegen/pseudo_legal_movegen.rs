
struct Move{
    from: usize,
    to: usize,
    promotion: usize,
    capture: usize,
    flags: usize
}

// const sliding: [bool;5] = [false, true, true, true, false]; //knight bishop rook queen king 
//braucht man denke nicht wenn man jedes piece sowieso seperat behandelt aber vllt nicht jedes piece seperat behandeln sondern
// nur in sliding und nicht sliding unterscheiden /  und halt in pawn moves am anfang sowieso 
const knight_offsets:[i8; 8] = [];
const bishop_offsets:[i8; 4] = [];
const rook_offsets:[i8; 4] = [];
const king_queen_offsets:[i8; 8] = [];

fn movegen(board : board) -> Vec<Move> {
    for (index, piece) in board.iter().enumerate() { 
    if let Some(piece) = &board[piece] {
        if piece.color == color_to_move {
    
        match piece.kind {
            PieceType::knight => { for offset in knight_offsets {
                if board[index + offset] == None {
                    //push move to vector
                }else if board[index + offset].color == color_to_move.opposite() {
                    //push move to vector with capture flag
                }
            }}
            PieceType::bishop => {
                for offset in bishop_offsets {
                    target = index + offset;
                    while board[target] != OFFBOARD {
                        if target == None {
                            // push move 
                            target += offset;
                        }else if board[target].color == color_to_move.opposite(){
                            //push move with capture
                            break;
                        }
                }
            }
        } PieceType::pawn => {
            if piece.two_movep == true && board[index+10]== None && board[index+20]== None {
                //push move tp vector with flag 
                //set two_movep to false
            }
            if board[index+10]== None {
                //push move to vector
            }
            if board[index+1].color == color_to_move.opposite &&  movecounter - board[index+1].two_move-counter <= 1 {

            }
            if board[index-1].color == color_to_move.opposite &&  movecounter - board[index-1].two_move-counter <= 1 {

        }
    }
        }}
}
    }
}