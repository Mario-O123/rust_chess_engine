//here we will handle the dataset loading

use crate::trainer_rust::config::positions_path;
use crate::trainer_rust::decode_fen::decode_data;
use burn::data::dataloader::batcher::Batcher;
use burn::data::dataloader::{DataLoader, DataLoaderBuilder};
use burn::data::dataset::Dataset;
use burn::tensor::Tensor;
use burn::tensor::backend::Backend;
use rand::seq::index::sample;
use rand::thread_rng;
use serde_json::Value;
use std::clone;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::Arc;

#[derive(Clone)]
pub struct ChessDataset {
    pub positions: Vec<[f32; 768]>,
    pub evals: Vec<f32>,
}

//important the B stands for backend
impl Dataset<([f32; 768], f32)> for ChessDataset {
    fn len(&self) -> usize {
        return self.positions.len();
    }

    fn get(&self, index: usize) -> Option<([f32; 768], f32)> {
        //so this returns
        if index >= self.len() {
            None
        } else {
            return Some((self.positions[index], self.evals[index]));
        }
    }
}

pub struct ChessBatcher;

#[derive(Debug, Clone)]
pub struct ChessBatch<B: Backend> {
    pub positions: Tensor<B, 2>, // [batch_size, 768]
    pub evals: Tensor<B, 1>,     // [batch_size]
}
//now we need to implement a batcher to make a vector of tuples into tensors?
impl<B: Backend> Batcher<B, ([f32; 768], f32), ChessBatch<B>> for ChessBatcher {
    fn batch(
        &self,
        items: Vec<([f32; 768], f32)>,
        device: &<B as Backend>::Device,
    ) -> ChessBatch<B> {
        let batch_positions_vec: Vec<[f32; 768]> = items.iter().map(|(x, _)| *x).collect();
        let flat_positions_vec: Vec<f32> = batch_positions_vec
            .iter()
            .flat_map(|arr| arr.iter())
            .copied()
            .collect();

        let batch_evals_vec: Vec<f32> = items.iter().map(|(_, y)| *y).collect();

        let batch_positions_1d: Tensor<B, 1> =
            Tensor::<B, 1>::from_floats(&flat_positions_vec[..], device);
        let batch_positions = batch_positions_1d.reshape([items.len(), 768]);

        let batch_evals: Tensor<B, 1> = Tensor::from_floats(&batch_evals_vec[..], device);

        ChessBatch {
            positions: batch_positions,
            evals: batch_evals,
        }
    }
}

//we also need to create a Dataloader?
pub fn create_dataloader<B: Backend>(
    dataset: Arc<ChessDataset>,
) -> Arc<dyn DataLoader<B, ChessBatch<B>>> {
    //here we can change the batch size the shuffle etc (shuffle parameter is a seed for shuffling?)
    return DataLoaderBuilder::new(ChessBatcher)
        .batch_size(256)
        .shuffle(597657)
        .build(dataset);
}
pub fn create_valid_dataloader<B: Backend>(
    dataset: ChessDataset,
) -> Arc<dyn DataLoader<B, ChessBatch<B>>> {
    return DataLoaderBuilder::new(ChessBatcher)
        .batch_size(256)
        .build(dataset);
}

pub fn load_dataset(path: &str, batch_size: usize) -> ChessDataset {
    let dataset_size = 102_653_432;
    let batch_size = batch_size;
    let mut rng = thread_rng();
    let indices = sample(&mut rng, dataset_size, batch_size);
    let hash_indices: HashSet<usize> = indices.iter().collect();

    //first we open the file and store it in the variable
    let file = File::open(positions_path).unwrap();
    //then we give the variable into the reader
    let reader = BufReader::new(file);

    let mut positions_x = Vec::with_capacity(batch_size);
    let mut evals_y = Vec::with_capacity(batch_size);

    //here we take each position from the dataset that is in our specified range and append it and its eval to the corresponding vectors
    for (index, line) in reader.lines().enumerate() {
        if !hash_indices.contains(&index) {
            continue;
        };

        let line = line.unwrap();
        let data: serde_json::Value = serde_json::from_str(&line).unwrap(); //load the json here

        //at best_eval as_array returns option so unwrap for that and max_by_key also returns option
        let fen = data.get("fen").and_then(Value::as_str).unwrap();
        let best_eval = data
            .get("evals")
            .and_then(Value::as_array)
            .unwrap()
            .iter()
            .max_by_key(|x| x.get("depth").and_then(Value::as_i64).unwrap_or(0))
            .unwrap();
        let best_pv = &best_eval
            .get("pvs")
            .and_then(Value::as_array)
            .and_then(|pvs| pvs.get(0))
            .unwrap();
        let label: f32;

        if best_pv.get("cp").is_some() {
            label = best_pv.get("cp").and_then(Value::as_f64).unwrap() as f32 / 1000.0;
        } else {
            //maybe mate as integer instead of float need to check in the dataset?

            let mate = best_pv.get("mate").and_then(Value::as_i64).unwrap() as i32;

            if mate > 0 {
                label = 10.0;
            } else {
                label = -10.0;
            }
        }
        positions_x.push(decode_data(fen));
        evals_y.push(label);

        if positions_x.len() == batch_size {
            break;
        }
    }
    return ChessDataset {
        positions: positions_x,
        evals: evals_y,
    };
}
