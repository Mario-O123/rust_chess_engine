//here we will write the function which will decode the fen data from the training data into the informations we want

pub fn decode_data(fen: &str) -> [f32; 768] {

    let board = fen.split_whitespace().next().unwrap();
    let mut features  = [0.0f32 ; 12*64];

    let mut square : usize = 0;
    for character in board.chars() {
        if character == '/'{
            continue;
        }
        else if character.is_ascii_digit(){
            square+= character.to_digit(10).unwrap() as usize;
        }
        else {
            
            let index = decode_pieces(character).unwrap() * 64 + square;
            features[index] = 1.0;
            square +=1;
        }
    }
    return features; 
    
}


fn decode_pieces(char : char) -> Option<usize>{
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