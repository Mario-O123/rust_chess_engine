

//here we call the functions of the other files in the directory
use burn::backend::{Wgpu, Autodiff};
use burn::module::Module;
use burn::record::FileRecorder;





use crate::trainer_rust::dataset::{load_dataset, create_dataloader};
use crate::trainer_rust::mlp_structure::MLP;
use crate::trainer_rust::train::train;
use crate::trainer_rust::config::{positions_path, batch_end , batch_start, model_path};
use burn::record::PrettyJsonFileRecorder;
use burn::record::FullPrecisionSettings;


pub fn main() {
type B = Autodiff<Wgpu<f32, i32>>;

let device = Default::default();

let recorder:PrettyJsonFileRecorder<FullPrecisionSettings> = PrettyJsonFileRecorder::new();



println!("Loading Dataset");
let dataset = load_dataset(positions_path, batch_start, batch_end);
let dataloader = create_dataloader::<B>(dataset);

//if there is a trained modelavailable we use th line below the model initialization if not then we comment it out
let model = MLP::<B>::new(768, 512, 256, &device);
//model= model.load_file(fmodel_path, &recorder, &device).unwrap();

let trained_model = train::<B>(model, dataloader);

trained_model.save_file(model_path, &recorder).expect("Error in saving model");
println!("Finished!");

}

//first call load_dataset for loading the training data

//then decode the fen from the data into NNUE readable format

//then load the MLP model and give it the parameters

//then also load the previous checkpoint? if there is one


//then write the training loop (for epochs ...)

//then save the model checkpoints
