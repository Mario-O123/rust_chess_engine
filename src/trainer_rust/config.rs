//define constants here

use std::path::PathBuf;

pub const model_path: &str = "src/trainer_rust/models/mlp_checkpoint.json";
pub const positions_path: &str = "/home/emil/chess_datsets/lichess_db_eval.jsonl";

pub const batch_start: usize = 0;
pub const batch_end: usize = 1_000_000;
pub const valid_start: usize = 1_000_000;
pub const valid_end: usize = 1_200_000;

pub const CHUNKS_DIR: &str = "";
pub const chunk_size: usize = 0;

//gelernte position grade 2m-2.6m
