//here we call the functions of the other files in the directory
use burn::backend::{Autodiff, Wgpu};
use burn::module::Module;
use burn::record::FileRecorder;
use std::sync::Arc;

use crate::trainer_rust::config::{
    batch_end, batch_start, model_path, positions_path, valid_end, valid_start,
};
use crate::trainer_rust::dataset::{
    ChessDataset, create_dataloader, create_valid_dataloader, load_dataset,
};
use crate::trainer_rust::mlp_structure::MLP;
use crate::trainer_rust::train::train;
use burn::record::FullPrecisionSettings;
use burn::record::PrettyJsonFileRecorder;

pub fn main() {
    type B = Autodiff<Wgpu<f32, i32>>;

    let device = Default::default();

    let recorder: PrettyJsonFileRecorder<FullPrecisionSettings> = PrettyJsonFileRecorder::new();

    println!("Loading Dataset");
    let train_size = 1_000_000;
    let combined_datsets = load_dataset(positions_path, 1_200_000);
    let dataset = Arc::new(ChessDataset {
        positions: combined_datsets.positions[..train_size].to_vec(),
        evals: combined_datsets.evals[..train_size].to_vec(),
    });
    let val_dataset = ChessDataset {
        positions: combined_datsets.positions[train_size..].to_vec(),
        evals: combined_datsets.evals[train_size..].to_vec(),
    };
    let dataloader = create_dataloader::<B>(Arc::clone(&dataset));
    let val_dataloader = create_valid_dataloader::<B>(val_dataset);

    //if there is a trained modelavailable we use th line below the model initialization if not then we comment it out
    let model = MLP::<B>::new(768, 512, 256, &device);
    let model = model.load_file(model_path, &recorder, &device).unwrap();

    let trained_model = train::<B>(model, dataset, val_dataloader);

    println!("Finished!");
}

//first call load_dataset for loading the training data

//then decode the fen from the data into NNUE readable format

//then load the MLP model and give it the parameters

//then also load the previous checkpoint? if there is one

//then write the training loop (for epochs ...)

//then save the model checkpoints
