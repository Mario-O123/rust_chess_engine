//here a position is encoded into mlp understandable format
use crate::board::mailbox120::SQUARE120_TO_SQUARE64;
use crate::position::{Cell, Color, Piece, PieceKind, Position};

//similar to decode_fen but instead of fen takes our position struct

const WK: u8 = 0b0001;
const WQ: u8 = 0b0010;
const BK: u8 = 0b0100;
const BQ: u8 = 0b1000;

pub fn decode_pos_nn(position: &Position) -> [f32; 781] {
    let mut features = [0.0f32; 12 * 64 + 1 + 4 + 8];
    //decode the board into 12*64 = 768 neurons
    for (sq120, cell) in position.board.iter().enumerate() {
        let sq64 = SQUARE120_TO_SQUARE64[sq120];
        if sq64 < 0 {
            continue;
        }
        let row = sq64 / 8;
        let col = sq64 % 8;
        let sq64_flipped = (7 - row) * 8 + col;

        match cell {
            Cell::Empty => {}
            Cell::Piece(piece) => {
                let index = decode_pos_pieces(piece).unwrap() * 64 + sq64_flipped as usize;

                features[index] = 1.0;
            }
            _ => continue,
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
        if square_index_64 >= 0 {
            features[773 + ((square_index_64 % 8) as usize)] = 1.0;    
        }
        
    }

    return features;
}

fn decode_pos_pieces(piece: &Piece) -> Option<usize> {
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
    } else {
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

#[cfg(test)]
mod nn_alignment_tests {
    use super::decode_pos_nn;
    use crate::position::Position;
    use crate::trainer_rust::decode_fen::decode_data;

    /// Returns all indices set to 1 in a given 64-square piece plane
    fn ones_in_plane(features: &[f32; 781], plane: usize) -> Vec<usize> {
        let start = plane * 64;
        let end = start + 64;
        let mut out = Vec::new();
        for (i, &v) in features[start..end].iter().enumerate() {
            if (v - 1.0).abs() < 1e-6 {
                out.push(i);
            }
        }
        out
    }

    #[test]
    fn engine_matches_trainer_feature_indices() {
        // FEN: White king on a1, Black king on h8
        let fen = "7k/8/8/8/8/8/8/K7 w - - 0 1";
        let pos = Position::from_fen(fen).expect("FEN should parse");

        // Engine and trainer features
        let feat_engine = decode_pos_nn(&pos);
        let feat_trainer = decode_data(fen);

        // Compare all 12 piece planes
        for plane in 0..12 {
            let eng = ones_in_plane(&feat_engine, plane);
            let trn = ones_in_plane(&feat_trainer, plane);
            assert_eq!(eng, trn, "Piece plane {} does not match trainer", plane);
        }

        // Compare remaining neurons (player to move, castling, en passant)
        assert_eq!(&feat_engine[768..], &feat_trainer[768..], "Non-piece features mismatch");
    }

}
