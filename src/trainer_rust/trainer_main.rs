//here we call the functions of the other files in the directory
use burn::backend::{Autodiff , NdArray}; //both wgpu and ndarray loaded trained on both will remove in final version after done with training
use burn::module::Module;
use std::sync::Arc;

use crate::trainer_rust::config::{
     POSITIONS_PATH , MODEL_PATH_2 };
use crate::trainer_rust::dataset::{
    ChessDataset, create_valid_dataloader, load_dataset,
};
use crate::trainer_rust::mlp_structure::MLP;
use crate::trainer_rust::train::train;
use burn::record::FullPrecisionSettings;
use burn::record::PrettyJsonFileRecorder;

pub fn main() {
    //define backend and device to train on and recorder
    type B = Autodiff<NdArray<f32>>;
    
    let device = Default::default();

    let recorder: PrettyJsonFileRecorder<FullPrecisionSettings> = PrettyJsonFileRecorder::new();

    println!("Loading Dataset");
    //load the traning data and valid data
    let train_size = 5_300_000;
    //let combined_datsets = load_dataset(positions_path, 5_500_000); //used most but possible to get more positoins in 1 run
    let combined_datsets = load_dataset(POSITIONS_PATH, 6_500_000);
    let dataset = Arc::new(ChessDataset {
        positions: combined_datsets.positions[..train_size].to_vec(),
        evals: combined_datsets.evals[..train_size].to_vec(),
    });
    let val_dataset = ChessDataset {
        positions: combined_datsets.positions[train_size..].to_vec(),
        evals: combined_datsets.evals[train_size..].to_vec(),
    };
    //let dataloader = create_dataloader::<B>(Arc::clone(&dataset));
    let val_dataloader = create_valid_dataloader::<B>(val_dataset);
    println!("Starting training!");
    //initalize model and load state
    //if there is a trained modelavailable we use th line below the model initialization if not then we comment it out
    //try doing 781 256 128 32 and if thats better try removing one hiddenlayer and do maybe 781 256 32 or 256 64 because for nnue better to use less hidden layers?
    let mut model = MLP::<B>::new(781, 256 , 64 ,  &device);
    model = model.load_file(MODEL_PATH_2, &recorder, &device).unwrap();

    let _trained_model = train::<B>(model, dataset, val_dataloader, &device);

    println!("Finished!");
}

//first call load_dataset for loading the training data

//then decode the fen from the data into NNUE readable format(done in the load_dataset logic by calling decode_fen)

//then initialize and load the MLP model and give it the parameters

//then also load the previous checkpoint? if there is one

//then write the training loop (for epochs ...)

//then save the model checkpoints (now done in the training loop for improved valid loss or at the start because new positions = better generalization either way)
