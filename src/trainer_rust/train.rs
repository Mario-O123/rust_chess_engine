//ok so here we define the training loop

//we need to load the optimizer and LossFunction

//then we need to iterate over the epochs and do the optmizer reset run the model calculate the loss and gradients then update with the optimizer in the loop for the batch
//then after give back the loss for each epoch

use burn::data::dataloader::{DataLoader, DatasetIterator};
use burn::optim::{AdamW, AdamWConfig, GradientsParams};
use burn::nn::loss::MseLoss;
use burn::prelude::ToElement;
use burn::tensor::Tensor;
use burn::tensor::backend::{AutodiffBackend, Backend};
use crate::trainer_rust::mlp_structure::MLP;
use crate::trainer_rust::dataset::ChessBatch;
use burn::nn::loss::Reduction;
use std::sync::Arc;
use burn::optim::Optimizer;


pub fn train<B : AutodiffBackend>(mut model : MLP<B> , loader : Arc<dyn DataLoader<B, ChessBatch<B>>>) -> MLP<B>
  {

    let  optimizer_config = AdamWConfig::new().with_weight_decay(1e-2);
    let mut optimizer = optimizer_config.init();
    let  loss_function = MseLoss::new();

    for epoch in 0..25 {
        let mut epoch_loss : f32 = 0.0;
        for batch in loader.iter() {

            let prediction = model.forward(batch.positions.clone());
            let evals = batch.evals.clone().unsqueeze();
            let loss_tensor = loss_function.forward(prediction, evals, Reduction::Auto);
            let loss_value : f32 = loss_tensor.clone().into_scalar().to_f32();
            let grad = loss_tensor.backward();
            let grad = GradientsParams::from_grads(grad, &model);
            model = optimizer.step(1e-3 , model, grad);

            epoch_loss += loss_value;

        }
        //here need to check if loader.len() gives back the number of batches because thats what we need
        let average_epoch_loss = epoch_loss / loader.iter().count() as f32;
        println!("Epoch: {}     Loss: {}" , epoch ,average_epoch_loss );
        
    }
   return model;



}