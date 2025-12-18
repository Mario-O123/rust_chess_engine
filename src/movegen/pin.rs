//Pin detection and X-Ray Attacks
//pinned if piece cant move without having own king in check

use crate::position::{Position, Color, PieceKind};
use crate::board::mailbox120::{
    is_on_board,
    ROOK_DIRECTIONS,
    BISHOP_DIRECTIONS,
};

//Pin representation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pin {
    pub pinned_square: usize,
    pub pinner_square: usize,
    pub kin_square: usize,
    pub direction: i8,
}

//finds all pins for given color 
//returns vec of all pins
pub fn find_all_pins(position: &Position, color: Color) -> Vec<Pin> {
    let mut pins = Vec::new();

    let king_square = match find_king(position, color) {
        Some(square) => square,
        None =>return pins,
    };

    let enemy_color = color.opposite();

    //checks Rook
    for &direction in &ROOK_DIRECTIONS {
        if let Some(pin) = check_pin_in_direction (
            position,
            king_square,
            direction,
            color,
            enemy_color,
            true
        )  {
            pins.push(pin);
        }       
        
    }

    //checks Bishops
    for &direction in &BISHOP_DIRECTIONS {
        if let Some(pin) = check_pin_in_direction (
            position,
            king_square,
            direction,
            color,
            enemy_color,
            false
        )  {
            pins.push(pin);
        }   
    }

    pins
}

//true if a piece is pinned
pub fn is_piece_pinned(position: &Position, square120: usize, color: Color) -> bool {
    let pins = find_all_pins(position, color);
    pins.into_iter().any(|pin| pin.pinned_square == square120)
}

//returns pin if theres a pinned piece
pub fn get_pin(position: &Position, square120: usize, color: Color) -> Option<Pin> {
    let pins = find_all_pins(position, color);
    pins.into_iter().find(|pin| pin.pinned_square == square120)
}

//checks if move is legal if there is pinned piece
pub fn is_move_legal_if_pinned(pin: &Pin, from: usize, to: usize) -> bool {
    if to == pin.pinned_square {
        return true;
    }

    is_on_pin_line(from, to, pin.king_square, pin.direction)
}

//Helperfunction: checks if square on pin-line
fn is_on_pin_line(from: usize, to: usize, king_square: usize, pin_direction: i8) -> bool {
    // checks if from->to same direction as pin direction
    let diff = (to as i32 - from as i32) as i32;
    if diff == 0 {
        return false;
    }
    
    // Normalize both 
    let pin_dir_norm = normalize_direction(pin_direction);
    let move_dir_norm = normalize_direction(diff);
    
    pin_dir_norm == move_dir_norm || pin_dir_norm == -move_dir_norm
}

//Normalizing direction
fn normalize_direction(dir: i32) -> i8 {
    if dir == 0 {
        return 0;
    }

    let sign = dir.signum();
    let abs = dir.abs();

    //Horizontal +-1
    //Vertical +- 10
    //diagonal +-9 , +-11

    if abs % 1 == 0 && abs < 10 {
        return sign as i8;
    } else if abs % 10 == 0 {
        return (10 * sign) as i8; // Vertikal (normalisiert auf ±1 in Richtung)
    } else if abs % 9 == 0 {
        return (9 * sign) as i8; // Diagonal /
    } else if abs % 11 == 0 {
        return (11 * sign) as i8; // Diagonal \
    }
    
    return 0;
}

fn check_pin_in_direction(
    position: &Position,
    king_square: usize,
    direction: i8,
    friendly_color: Color,
    enemy_color: Color,
    is_rook_direction: bool, // true = Rook/Queen, false = Bishop/Queen
) -> Option<Pin> {

}

pub fn find_xray_attackers(
    position: &Position,
    target_square: usize,
    by_color: Color,
) -> Vec<usize> {

}

fn find_xray_in_direction(
    position: &Position,
    target_square: usize,
    direction: i8,
    by_color: Color,
    is_rook_direction: bool,
) -> Option<usize> {

}



