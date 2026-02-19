mod terminal_common;

use rust_chess_engine::evaluation::neural::NeuralEval;

fn main() -> anyhow::Result<()> {
    let default_path = format!(
        "{}/src/trainer_rust/models/mlp_checkpoint_3.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let model_path = std::env::args().nth(1).unwrap_or(default_path);

    let eval_for_search = NeuralEval::load(&model_path)?;
    let eval_view = NeuralEval::load(&model_path)?;

    terminal_common::run_repl(eval_for_search, eval_view);
    Ok(())
}
