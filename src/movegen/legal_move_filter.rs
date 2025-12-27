


fn legal_move_filter(position: &Position , moves : Vec<Moves>) -> Vec<Move> {

    let mut legal_moves = Vec::new();    

    for pseudo_move in moves {
        position.make_move();

        if !is_in_check(position, position.player_to_move) {
            legal_moves.push(pseudo_move)
        }

        position.unmake_move()
    }
    return legal_moves;
}

