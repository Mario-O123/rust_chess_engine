//ok so here we define the training loop

//we need to load the optimizer and LossFunction

//then we need to iterate over the epochs and do the optmizer reset run the model calculate the loss and gradients then update with the optimizer in the loop for the batch
//then after give back the loss for each epoch

use crate::trainer_rust::config::{batch_end, batch_start, model_path, positions_path};
use crate::trainer_rust::dataset::{ChessBatch, ChessBatcher, ChessDataset, load_dataset};
use crate::trainer_rust::mlp_structure::MLP;
use burn::data::dataloader::{DataLoader, DataLoaderBuilder, DatasetIterator};
use burn::data::{self, dataset};
use burn::module::AutodiffModule;
use burn::module::Module;
use burn::nn::loss::MseLoss;
use burn::nn::loss::Reduction;
use burn::optim::Optimizer;
use burn::optim::{AdamW, AdamWConfig, GradientsParams};
use burn::prelude::ToElement;
use burn::record::FileRecorder;
use burn::record::FullPrecisionSettings;
use burn::record::PrettyJsonFileRecorder;
use burn::tensor::Tensor;
use burn::tensor::backend::{AutodiffBackend, Backend};
use std::num;
use std::sync::Arc;

pub fn train<B: AutodiffBackend>(
    mut model: MLP<B>,
    dataset: Arc<ChessDataset>,
    val_loader: Arc<dyn DataLoader<B, ChessBatch<B>>>,
) -> MLP<B> {
    let mut best_val_loss: f32 = 1000.0;
    let optimizer_config = AdamWConfig::new().with_weight_decay(2e-5);
    let mut optimizer = optimizer_config.init();

    let loss_function = MseLoss::new();

    for epoch in 0..20 {
        let mut epoch_loss: f32 = 0.0;
        let mut batch_num = 0;
        let loader = DataLoaderBuilder::new(ChessBatcher)
            .batch_size(256)
            .shuffle(epoch as u64) // new seed each epoch
            .build(dataset.clone());
        for batch in loader.iter() {
            let prediction = model.forward(batch.positions);
            let evals = batch.evals.unsqueeze_dim(1);
            let loss_tensor = loss_function.forward(prediction, evals, Reduction::Mean);
            let loss_value: f32 = loss_tensor.clone().into_scalar().to_f32();
            let grad = loss_tensor.backward();
            let grads = GradientsParams::from_grads(grad, &model);
            batch_num += 1;
            //change lr to ~5e-4 when new training runs arent are jumping to high then to 1e-4 and so on till mybe 1e-5
            model = optimizer.step(2e-4, model, grads);

            epoch_loss += loss_value;
        }
        let valid_model = model.valid();
        let mut valid_loss = 0.0;
        for batch in val_loader.iter() {
            let val_pred = valid_model.forward(batch.positions.clone().inner());
            let val_evals = batch.evals.inner().unsqueeze_dim(1);
            let val_loss_tensor = loss_function.forward(val_pred, val_evals, Reduction::Mean);
            let val_loss_value: f32 = val_loss_tensor.clone().into_scalar().to_f32();
            valid_loss += val_loss_value;
        }

        //here need to check if loader.len() gives back the number of batches because thats what we need
        let average_epoch_loss = epoch_loss / batch_num as f32;
        let average_valid_loss = valid_loss / batch_num as f32;
        let real_loss = average_epoch_loss.sqrt();
        println!("Train - Epoch: {}     Loss: {}", epoch, average_epoch_loss);
        println!("Valid - Epoch: {}     Loss: {}", epoch, average_valid_loss);
        if average_valid_loss < best_val_loss {
            best_val_loss = average_valid_loss;
            let best_model = model.clone(); // keep the best
            let recorder: PrettyJsonFileRecorder<FullPrecisionSettings> =
                PrettyJsonFileRecorder::new();

            best_model
                .save_file(model_path, &recorder)
                .expect("Error in saving model");
            println!("Still Fine!");
        }
    }
    return model;
}
