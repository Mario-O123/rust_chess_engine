use crate::mailbox120::{
    SQUARE64_TO_SQUARE120, 
    SQUARE120_TO_SQUARE64, 
    OFFBOARD,
    ROOK_DIRECTIONS,
    BISHOP_DIRECTIONS,
    QUEEN_DIRECTIONS,
    KNIGHT_DIRECTIONS,
    BOARD_SIZE,
    square120_from_file_rank,
    is_on_board,
};

//extract File and Rank out ouf 120er Index
#[inline]
pub fn file_rank_from_square120(square120: usize) -> Option<(u8, u8)> {
    if (!is_on_board(square120)) {
        println!("ERROR: square120 not on Board!");
        None
    } else {
        let adjusted = square120 - 21;
        let file = (adjusted % 10) as u8;
        let rank = (adjusted / 10) as u8;
        Some(file, rank)
    }
    
}

//Algebraic notation of square120 (example: "e4")
#[inline]
pub fn square120_to_string(square120: usize) -> Option<String> {
    if !is_on_board(square120) {
        return None;
    }
    let (file, rank) = file_rank_from_square120(square120);
    let file_char = (b'a' + file) as char;
    let rank_char = (b'1' + rank) as char;
    Some(format!("{}{}", file_char, rank_char))
}

//Convert Piece to Char
//White = positiv + Uppercase, Black =  negativ + Lowercase
#[inline]
pub fn piece_to_char(piece: i8) -> char {
    match piece {
        1 => 'P',
        2 => 'N',
        3 => 'B',
        4 => 'R',
        5 => 'Q',
        6 => 'K',
        -1 => 'p',
        -2 => 'n',
        -3 => 'b',
        -4 => 'r',
        -5 => 'q',
        -6 => 'k',
        _ => '.',
    }
}

//piece to char unicode
//White -> outlines, Black -> full
#[inline]
pub fn piece_to_char_unicode(p: i8) -> char {
    match p {
        1  => '♙',  
        2  => '♘',  
        3  => '♗',  
        4  => '♖',  
        5  => '♕',  
        6  => '♔',  
        -1 => '♟',  
        -2 => '♞',  
        -3 => '♝',  
        -4 => '♜', 
        -5 => '♛', 
        -6 => '♚', 
        _  => '·',  
    }
}