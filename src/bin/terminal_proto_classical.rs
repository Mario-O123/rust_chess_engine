mod terminal_common;

use rust_chess_engine::evaluation::{ClassicalEval};

fn main() {
    terminal_common::run_repl(ClassicalEval::new(), ClassicalEval::new());
}


#[cfg(test)]
mod terminal_promo_cli_tests {
    use crate::terminal_common::parse_go_limits;
    use rust_chess_engine::search::SearchLimits;

    fn default_limits() -> SearchLimits {
        SearchLimits {
            max_depth: 5,
            max_nodes: None,
            max_time_ms: None,
        }
    }
    #[test]
    fn empty_go_tokens_returns_defaults() {
        let limits = parse_go_limits(&[], default_limits());
        assert_eq!(limits.max_depth, 7);
        assert_eq!(limits.max_time_ms, Some(2000));
        assert_eq!(limits.max_nodes, None);
    }

    #[test]
    fn depth_overrides_default() {
        let limits = parse_go_limits(&["depth", "7"], default_limits());
        assert_eq!(limits.max_depth, 7);
        assert_eq!(limits.max_time_ms, Some(2000));
        assert_eq!(limits.max_nodes, None);
    }

    #[test]
    fn depth_zero_is_raised_to_one() {
        let limits = parse_go_limits(&["depth", "0"], default_limits());
        assert_eq!(limits.max_depth, 1);
    }

    #[test]
    fn time_and_nodes_override_defaults() {
        let limits = parse_go_limits(&["time", "1000", "nodes", "200000"], default_limits());
        assert_eq!(limits.max_depth, 5);
        assert_eq!(limits.max_time_ms, Some(2000));
        assert_eq!(limits.max_nodes, Some(200000));
    }

    #[test]
    fn unknown_tokens_and_invalid_values_are_ignored() {
        let limits = parse_go_limits(
            &["random", "123", "depth", "abc", "time", "xyz", "nodes", "-1"],
            default_limits(),
        );
        
        assert_eq!(limits.max_depth, 7);
        assert_eq!(limits.max_time_ms, Some(2000));
        assert_eq!(limits.max_nodes, None);
    }
}

#[cfg(test)]
mod terminal_promo_handle_line_tests {
    use crate::terminal_common::EngineCli;
    use rust_chess_engine::evaluation::ClassicalEval;

    fn fresh_cli() -> EngineCli<ClassicalEval> {
    EngineCli::new(ClassicalEval::new(), ClassicalEval::new())
    }

    #[test]
    fn new_resets_the_game() {
        let mut cli = EngineCli::new(ClassicalEval::new(), ClassicalEval::new());

        cli.handle_line("engine off");
        cli.handle_line("e2e4");

        let fen_after_move = cli.game.position().to_fen();
        assert_ne!(fen_after_move, fresh_cli().game.position().to_fen());

        cli.handle_line("new");
        let fen_after_new = cli.game.position().to_fen();

        let fen_start = fresh_cli().game.position().to_fen();
        assert_eq!(fen_after_new, fen_start);
    }

    #[test]
    fn engine_toggle_off_on_does_not_quit() {
        let mut cli = fresh_cli();
        assert!(!cli.handle_line("engine off"));
        assert!(!cli.handle_line("engine on"));
    }

    #[test]
    fn legal_user_move_changes_position_when_engine_off() {
        let mut cli = fresh_cli();
        cli.handle_line("engine off");

        let fen_before = cli.game.position().to_fen();
        let side_before = cli.game.position().player_to_move;

        assert!(!cli.handle_line("e2e4"));

        let fen_after = cli.game.position().to_fen();
        let side_after = cli.game.position().player_to_move;

        assert_ne!(fen_after, fen_before);
        assert_ne!(side_after, side_before);
    }

    #[test]
    fn illegal_user_move_does_not_change_position() {
        let mut cli = fresh_cli();
        cli.handle_line("engine off");

        let fen_before = cli.game.position().to_fen();
        let side_before = cli.game.position().player_to_move;

        assert!(!cli.handle_line("e2e1"));

        let fen_after = cli.game.position().to_fen();
        let side_after = cli.game.position().player_to_move;

        assert_eq!(fen_after, fen_before);
        assert_eq!(side_after, side_before);
    }
}





