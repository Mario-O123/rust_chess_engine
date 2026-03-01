//define constants here like the model and optimizer paths

pub const MODEL_PATH: &str = "src/trainer_rust/models/mlp_checkpoint_2.json";
pub const POSITIONS_PATH: &str = "D:/127/uni/chessdata/lichess_db_eval.jsonl";
pub const OPTIMIZER_SAVE_PATH: &str = "src/trainer_rust/models/optimizer_checkpoint_2.json";

pub const MODEL_PATH_2: &str = "src/trainer_rust/models/mlp_checkpoint_3.json";
pub const OPTIMIZER_SAVE_PATH_2: &str = "src/trainer_rust/models/optimizer_checkpoint_4.json";

pub const MODEL_PATH_4: &str = "src/trainer_rust/models/mlp_checkpoint_4.json";
pub const OPTIMIZER_SAVE_PATH_4: &str = "src/trainer_rust/models/optimizer_checkpoint_4.json";

//number of positions in the training dataset we pull our data from (https://database.lichess.org/#evals) (CC0 license so free to use for us)
//file lines : 342059879
