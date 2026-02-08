//here we will write the function which will decode the fen data from the training data into the informations we want

pub fn decode_data(fen: &str) -> [f32; 781] {
    //split the fen into the features we want to decode (board , movecolor , castling rights , enpassant square)
    let parts : Vec<&str>= fen.split_whitespace().collect();
    let board = parts[0];
    let move_color = parts[1];
    let castle_rights = parts[2];
    let en_passant_sqr = parts[3];
    let mut features = [0.0f32; 12 * 64 +1 + 4 + 8 ];

    let mut square: usize = 0;
    //decode the fen by first going over the board and if a piece is on a index we put a 1 on the index in the 64 vec thats designated for the piece (12 total for every piece of both colors (2*6) so 12*64 neurons)
    for character in board.chars() {
        if character == '/' {
            continue;
        } else if character.is_ascii_digit() {
            square += character.to_digit(10).unwrap() as usize;
        } else {
            let index = decode_pieces(character).unwrap() * 64 + square;
            features[index] = 1.0;
            square += 1;
        }
    }
    //one neuron is for the color which has the turn
    if move_color == "w" {
        features[768] = 1.0;
    }else {
        features[768] = 0.0
    }
    //4 neurons for the castling rights 
    if castle_rights != "-" {
        for character in castle_rights.chars() {
            match character {
                'k' => features[769] = 1.0,
                'q' => features[770] = 1.0,
                'K' => features[771] = 1.0,
                'Q' => features[772] = 1.0,
                 _  => {}
            }
        }
    }
    //then another 8 neurons for the file which has the ep square (the rank is implicated because ep can only be done after a pawn does a double move so we dont need to add neurons for that)
    if en_passant_sqr != "-" {
        let file = (en_passant_sqr.as_bytes()[0] - b'a') as usize;
        features[773+file] = 1.0;
}
    return features;
}
//here we give each piece the index of ots 64 vec inside the total 768 neuon vec for example white pawn (P) has the first 64 neurons and white knight(N) the second 64 neurons
fn decode_pieces(char: char) -> Option<usize> {
    match char {
        'P' => Some(0),
        'N' => Some(1),
        'B' => Some(2),
        'R' => Some(3),
        'Q' => Some(4),
        'K' => Some(5),
        'p' => Some(6),
        'n' => Some(7),
        'b' => Some(8),
        'r' => Some(9),
        'q' => Some(10),
        'k' => Some(11),
        _ => None,
    }
}
