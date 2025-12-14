//Helperfunctions to print and check the Board
use crate::mailbox120::{
    SQUARE64_TO_SQUARE120, 
    SQUARE120_TO_SQUARE64, 
    OFFBOARD,
    ROOK_DIRECTIONS,
    BISHOP_DIRECTIONS,
    QUEEN_DIRECTIONS,
    KNIGHT_DIRECTIONS,
    square120_from_file_rank,
};


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

//extract File and Rank out ouf 120er Index
#[inline]
pub fn file_rank_from_square120(square120: usize) -> (u8, u8) {
    let adjusted = square120 - 21;
    let file = (adjusted % 10) as u8;
    let rank = (adjusted / 10) as u8;
    (file, rank)
}

// prints file_rank table
pub fn print_file_rank_table() {
    for rank in 0..8 {
        for file in 0..8 {
            let square120 = square120_from_file_rank(file, rank);
            println!("file {file}, rank {rank} -> {square120}");
        }
    }
}

//Algebraic notation of square120 (example: "e4")
#[inline]
pub fn square120_to_string(square120: usize) -> Option<String> {
    if SQUARE120_TO_SQUARE64[square120] == OFFBOARD {
        return None;
    }
    let (file, rank) = file_rank_from_square120(square120);
    let file_char = (b'a' + file) as char;
    let rank_char = (b'1' + rank) as char;
    Some(format!("{}{}", file_char, rank_char))
}

//print mailbox120 structure OFFBOARD inclusive
pub fn print_mailbox120_structure() {
    println!("========== MAILBOX 120 STRUCTURE ==========");
    println!("(## = OFFBOARD, Numbers = Valid square120)");
    println!();
    
    for row in 0..12 {
        print!("Row {:2}: ", row);
        for col in 0..10 {
            let square120 = row * 10 + col;
            if SQUARE120_TO_SQUARE64[square120] == OFFBOARD {
                print!(" ##");
            } else {
                print!("{:3}", square120);
            }
        }
        println!();
    }
    
    println!();
    println!("===========================================");
}


//checks consistency 64<->120 mapping
pub fn debug_check_mapping_square64_square120() {
    println!("========== MAPPING CONSISTENCY CHECK ==========");
    let mut errors = 0;

    // Test 1: 64 -> 120 -> 64 Round-Trip
    for square64 in 0..64 {
        let square120 = SQUARE64_TO_SQUARE120[square64];
        let square64_back = SQUARE120_TO_SQUARE64[square120];
        
        if square64_back != square64 as i8 {
            println!("ERROR: square64={} -> square120={} -> square64={}", 
                square64, square120, square64_back);
            errors += 1;
        }
    }

    // Test 2: 120 -> 64 -> 120 Round-Trip (nur valide Felder)
    for square120 in 0..120 {
        if SQUARE120_TO_SQUARE64[square120] == OFFBOARD {
            continue;
        }
        
        let square64 = SQUARE120_TO_SQUARE64[square120] as usize;
        let square120_back = SQUARE64_TO_SQUARE120[square64];
        
        if square120_back != square120 {
            println!("ERROR: square120={} -> square64={} -> square120={}", 
                square120, square64, square120_back);
            errors += 1;
        }
    }

    // Test 3: Zähle valide Felder
    let valid_count = (0..120)
        .filter(|&square| SQUARE120_TO_SQUARE64[square] != OFFBOARD)
        .count();
    
    if valid_count != 64 {
        println!("ERROR: Found {} valid squares, expected 64", valid_count);
        errors += 1;
    }
    
    if errors == 0 {
        println!("All mapping checks passed!");
        println!("   - 64 valid squares found");
        println!("   - Round-trip consistency OK");
    } else {
        println!("Found {} errors in mapping!", errors);
    }
    println!("=============================================");
}



// gives information about a square120
pub fn debug_print_square120_info(square120: usize) {
    println!("========== SQUARE120 INFO ==========");
    println!("square120: {}", square120);
    
    if SQUARE120_TO_SQUARE64[square120] == OFFBOARD {
        println!("Status: OFFBOARD");
    } else {
        let square64 = SQUARE120_TO_SQUARE64[square120] as u8;
        let (file, rank) = file_rank_from_square120(square120);
        let file_char = (b'a' + file) as char;
        let rank_char = (b'1' + rank) as char;
        
        println!("Status: Valid");
        println!("square64: {}", square64);
        println!("Algebraic: {}{}", file_char, rank_char);
        println!("File: {} ({})", file, file_char);
        println!("Rank: {} ({})", rank, rank_char);
    }
    println!("====================================");
}


// gives all possible moves of a square120 with an offset array
pub fn debug_print_offset_moves(square120: usize, offsets: &[i8], piece_name: &str) {
    println!("========== OFFSET MOVES DEBUG ==========");
    println!("Piece: {}", piece_name);
    println!("From square120: {}", square120);
    
    if SQUARE120_TO_SQUARE64[square120] == OFFBOARD {
        println!("ERROR: Starting square is OFFBOARD!");
        println!("========================================");
        return;
    }
    
    let (file, rank) = file_rank_from_square120(square120);
    let file_char = (b'a' + file) as char;
    let rank_char = (b'1' + rank) as char;
    println!("From: {}{} (file={}, rank={})", file_char, rank_char, file, rank);
    println!();
    
    println!("Possible moves with offsets:");
    for (i, &offset) in offsets.iter().enumerate() {
        let target = (square120 as i8 + offset) as usize;
        
        if target >= 120 {
            println!("  [{}] offset {:3} -> OUT OF BOUNDS", i, offset);
            continue;
        }
        
        if SQUARE120_TO_SQUARE64[target] == OFFBOARD {
            println!("  [{}] offset {:3} -> square120={:3} OFFBOARD", i, offset, target);
        } else {
            let (t_file, t_rank) = file_rank_from_square120(target);
            let t_file_char = (b'a' + t_file) as char;
            let t_rank_char = (b'1' + t_rank) as char;
            println!("  [{}] offset {:3} -> square120={:3} ({}{})", 
                i, offset, target, t_file_char, t_rank_char);
        }
    }
    println!("========================================");
}


