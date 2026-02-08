//here a position is encoded into mlp understandable format
use crate::position::{Cell, Color, Piece, PieceKind, Position};
use crate::board::mailbox120::SQUARE120_TO_SQUARE64;

//similar to decode_fen but instead of fen takes our position struct



const WK : u8 = 0b0001;
const WQ : u8 = 0b0010;
const BK : u8 = 0b0100;
const BQ : u8 = 0b1000;



pub fn decode_pos_nn( position: &Position) -> [f32; 781]  {

let mut features = [0.0f32; 12 * 64 +1 + 4 + 8 ];
    //decode the board into 12*64 = 768 neurons
    for (sq120 , cell) in position.board.iter().enumerate() {
        let sq64 = SQUARE120_TO_SQUARE64[sq120];
        if sq64 < 0 { continue;}
        match cell {
            Cell::Empty => {},
            Cell::Piece(piece) => {
                let index = decode_pos_pieces(piece).unwrap() * 64 + sq64 as usize;
            
            features[index] = 1.0;
            
            }
            _ => continue
        }
        
        
    
    }
    //player to move in another neuron
    if position.player_to_move == Color::White {
        features[768] = 1.0;
    } 
    //castle rights in 4 neurons
    if position.castling_rights != 0 {
        if position.castling_rights & BK != 0 {
            features[769] = 1.0;
        }
        if position.castling_rights & BQ != 0 {
            features[770] = 1.0;
        }
        if position.castling_rights & WK != 0 {
            features[771] = 1.0;
        }
        if position.castling_rights & WQ != 0 {
            features[772] = 1.0;
        }
    }
    //en passant square in 8 neurons(1 for each file )
    if position.en_passant_square != None {
        let square_index = position.en_passant_square.unwrap().as_usize();
        let square_index_64 = SQUARE120_TO_SQUARE64[square_index];
        features[773 +((square_index_64 % 8) as usize)] = 1.0;

    }

    return features
}

fn decode_pos_pieces(piece :&Piece) -> Option<usize> {
    //here match the piece to the 64 sections of the 768 board neurons based on color and piecekind
    if piece.color == Color::White {
        match piece.kind {
            PieceKind::Pawn => Some(0),
            PieceKind::Knight => Some(1),
            PieceKind::Bishop => Some(2),
            PieceKind::Rook => Some(3),
            PieceKind::Queen => Some(4),
            PieceKind::King => Some(5),
        }
    }
    else {
         match piece.kind {
            PieceKind::Pawn => Some(6),
            PieceKind::Knight => Some(7),
            PieceKind::Bishop => Some(8),
            PieceKind::Rook => Some(9),
            PieceKind::Queen => Some(10),
            PieceKind::King => Some(11),
        }
    }

}