//ok so here we define the training loop

//we need to load the optimizer and LossFunction

//then we need to iterate over the epochs and do the optmizer reset run the model calculate the loss and gradients then update with the optimizer in the loop for the batch
//then after give back the loss for each epoch

use crate::trainer_rust::config::{MODEL_PATH_2, OPTIMIZER_SAVE_PATH_2};
use crate::trainer_rust::dataset::{ChessBatch, ChessBatcher, ChessDataset};
use crate::trainer_rust::mlp_structure::MLP;
use burn::data::dataloader::{DataLoader, DataLoaderBuilder};
use burn::module::AutodiffModule;
use burn::module::Module;
use burn::nn::loss::MseLoss;
use burn::nn::loss::Reduction;
use burn::optim::Optimizer;
use burn::optim::adaptor::OptimizerAdaptor;
use burn::optim::{Adam, AdamConfig, GradientsParams};
use burn::prelude::ToElement;
use burn::record::FullPrecisionSettings;
use burn::record::PrettyJsonFileRecorder;
use burn::record::Recorder;
use burn::tensor::backend::AutodiffBackend;
use std::io::{self, Write};
use std::path::Path;
use std::sync::Arc;

pub fn train<B: AutodiffBackend>(
    mut model: MLP<B>,
    dataset: Arc<ChessDataset>,
    val_loader: Arc<dyn DataLoader<B, ChessBatch<B>>>,
    device: &B::Device,
) -> MLP<B> {
    //initialize variable for later checking if val loss decreased
    let mut best_val_loss: f32 = 1000.0;
    //initialize optimizer
    let optimizer_config = AdamConfig::new();
    let mut optimizer = optimizer_config.init();
    //load optimizer state if it exists
    if Path::new(&OPTIMIZER_SAVE_PATH_2).exists() {
        let device = device; // get backend device
        let recorder: PrettyJsonFileRecorder<FullPrecisionSettings> = PrettyJsonFileRecorder::new();

        //load the  tje optimizer state in the record
        let optimizer_record = recorder
            .load::<<OptimizerAdaptor<Adam, MLP<B>, B> as Optimizer<MLP<B>, B>>::Record>(
                OPTIMIZER_SAVE_PATH_2.into(),
                device,
            )
            .expect("Failed to load optimizer record");

        // then we load the optimizer from the record
        optimizer = optimizer.load_record(optimizer_record);

        println!("Loaded optimizer from checkpoint.");
    }

    //initialie loss function and define learning rate of optimizer
    let loss_function = MseLoss::new();
    let mut lr = 2e-6;

    //loop over epochs (wont go 20 epochs but str c when overfitting)
    for epoch in 0..20 {
        let mut epoch_loss: f32 = 0.0;
        let mut batch_num = 0;
        let mut valid_batches = 0;
        //initialize the data loader and make randomly shuffled minibatches
        let loader = DataLoaderBuilder::new(ChessBatcher)
            .batch_size(32)
            .shuffle(epoch as u64) //give epoch as seed to every epoch data gets shuffled again for more randomness and generalizing
            .build(dataset.clone());

        //do the forward pass and loss , optimizer for each minibatch
        for batch in loader.iter() {
            //forward -> loss function -> get gradients -> optmizer
            let prediction = model.forward(batch.positions);
            let evals = batch.evals.unsqueeze_dim(1);
            let loss_tensor = loss_function.forward(prediction, evals, Reduction::Mean);
            let loss_value: f32 = loss_tensor.clone().into_scalar().to_f32();
            let grad = loss_tensor.backward();
            let grads = GradientsParams::from_grads(grad, &model);
            batch_num += 1;
            //change lr to ~5e-4 when new training runs arent are jumping too high then to 1e-4 and so on if overfitting or bigger batches?
            model = optimizer.step(lr, model, grads);

            //progress of the run
            if batch_num % 1000 == 0 {
                print!(
                    "\r\x1b[2KBatch {} - {:.2}% ",
                    batch_num,
                    (batch_num as f32 / 140_000.0) * 100.0
                );
                io::stdout().flush().unwrap();
            }
            epoch_loss += loss_value;
        }

        //here we do the valid run where we go over the dataset which wasnt trained on and see how good the model perfomrs
        //same as the train run but no optimizer just a forward pass and calculate the loss
        let valid_model = model.valid();
        let mut valid_loss = 0.0;
        for batch in val_loader.iter() {
            let val_pred = valid_model.forward(batch.positions.inner());
            let val_evals = batch.evals.inner().unsqueeze_dim(1);
            let val_loss_tensor = loss_function.forward(val_pred, val_evals, Reduction::Mean);
            let val_loss_value: f32 = val_loss_tensor.into_scalar().to_f32();
            valid_loss += val_loss_value;
            valid_batches += 1;
        }

        //calculate the loss and translate it into cp by scaling back and/or using atanh though that could be worse
        let average_epoch_loss = epoch_loss / batch_num as f32;
        let average_valid_loss = valid_loss / valid_batches as f32;
        //let real_loss = average_epoch_loss.sqrt();
        println!(
            "Train - Epoch: {}     Loss: {}    cp: {}",
            epoch,
            average_epoch_loss,
            average_epoch_loss.sqrt() * 600.0
        );
        println!(
            "Valid - Epoch: {}     Loss: {}    cp: {}, weird cp: {}",
            epoch,
            average_valid_loss,
            average_valid_loss.sqrt() * 600.0,
            average_valid_loss.sqrt().atanh() * 600.0
        );

        //if valid loss plateaus we decrease lr to counter overfitting
        if average_valid_loss > best_val_loss - 1e-4 {
            lr = lr * 0.5;
        }

        //save a model if it preforms better doesnt save overfitted models
        if average_valid_loss < best_val_loss {
            best_val_loss = average_valid_loss;
            let best_model = model.clone(); // keep the best
            let recorder: PrettyJsonFileRecorder<FullPrecisionSettings> =
                PrettyJsonFileRecorder::new();

            best_model
                .save_file(MODEL_PATH_2, &recorder)
                .expect("Error in saving model");

            let optimizer_record = optimizer.to_record();
            recorder
                .record(optimizer_record, OPTIMIZER_SAVE_PATH_2.into()) // Path can be whatever you want
                .expect("Failed to save optimizer");
            println!("Still Fine!");
        }
    }
    return model;
}
