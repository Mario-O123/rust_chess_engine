//here a board is given a score so calls fature.rs then the mlp structure from the trainer

use super::super::Evaluator;
use crate::evaluation::neural::feature::decode_pos_nn;
use crate::position::Position;
use crate::trainer_rust::mlp_structure::MLP;
use burn::module::Module;
use burn::record::FullPrecisionSettings;
use burn::record::PrettyJsonFileRecorder;
use burn::tensor::Tensor;
use burn::tensor::backend::Backend;

pub struct NeuralEval<B: Backend> {
    model: MLP<B>,
    device: B::Device,
}

impl<B: Backend> NeuralEval<B> {
    //loading our device recorder and model
    pub fn load(model_path: &str) -> Self {
        //type B = NdArrayDevice;//(CPU)
        let device = B::Device::default();
        let recorder: PrettyJsonFileRecorder<FullPrecisionSettings> = PrettyJsonFileRecorder::new();
        let mut model: MLP<B> = MLP::<B>::new(781, 256, 64, &device);
        model = model.load_file(model_path, &recorder, &device).unwrap();

        Self { model, device }
    }
    //deocdeing the Position struct into our neuron format
    fn encode(&self, position: &Position) -> Tensor<B, 2> {
        let nn_input = decode_pos_nn(position);
        let nn_input_tensor = Tensor::<B, 1>::from_floats(&nn_input[..], &self.device);
        let nn_input_tensor_shaped = nn_input_tensor.reshape([1, 781]);
        return nn_input_tensor_shaped;
    }
}

impl<B: Backend> Evaluator for NeuralEval<B> {
    //the pass into our mlp which returns a score
    fn evaluate(&mut self, position: &Position) -> i32 {
        let input = self.encode(position);
        let prediction = self.model.forward(input);
        let score: f32 = prediction.to_data().to_vec::<f32>().unwrap()[0];

        if score >= 1.2 {
            return 30_000;
        }
        if score <= -1.2 {
            return -30_000;
        }

        let cp_score: f32 = 600.0 * score.atanh(); //.atanh();
        return cp_score.clamp(-30_000.0, 30_000.0) as i32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use burn::backend::NdArray;

    #[test]
    fn nn_eval_starting_pos() {
        let model_path = "src/trainer_rust/models/mlp_checkpoint_3.json";

        let mut eval = NeuralEval::<NdArray>::load(model_path);

        //make starting pos
        let pos = Position::starting_position();

        //run the mlp forward pass on it
        let score = eval.evaluate(&pos);

        println!("score : {}", score);

        // check if no weird scores
        assert!(score > -30_000);
        assert!(score < 30_000);
    }

    //161 -> 173 -> 235 -> 185 -> 192 -> 227 -> 233 -> 219 -> 206 -> 253 -> 334 -> 254
    #[test]
    fn eval_random_pos() {
        let model_path = "src/trainer_rust/models/mlp_checkpoint_3.json";

        let mut eval = NeuralEval::<NdArray>::load(model_path);

        //get position from some fen
        let pos =
            Position::from_fen("2r1N3/2P5/1b1rkp2/B1P1Pp2/1R1p4/5N2/6B1/K7 w - - 0 1").unwrap();

        //run the forward
        let score = eval.evaluate(&pos);

        println!("score : {}", score);

        //see it it gives advantage for right colour
        assert!(score > 0);
    }
    //another fen test
    //716 -> 30000 -> 30000 -> 30000 -> 30000 -> 30000
    #[test]
    fn eval_random_pos_2() {
        let model_path = "src/trainer_rust/models/mlp_checkpoint_3.json";

        let mut eval = NeuralEval::<NdArray>::load(model_path);

        let pos =
            Position::from_fen("2B1k3/5N2/P5p1/BpK1p2R/2b1P3/5r2/P5P1/5n2 w - - 0 1").unwrap();

        let score = eval.evaluate(&pos);

        println!("score : {}", score);

        assert!(score > 0);
    }

    //r1b1k2r/p1p2ppp/2n2n2/qp2p3/2PP4/P4N2/2PB1PPP/R2QKB1R w KQkq - 0 10
    //278
    #[test]
    fn eval_random_pos_3() {
        let model_path = "src/trainer_rust/models/mlp_checkpoint_3.json";

        let mut eval = NeuralEval::<NdArray>::load(model_path);

        let pos =
            Position::from_fen("r3k2r/pn1b1p1p/6p1/1Pp1P3/6n1/P4N2/2P2PPP/R3KB1R w KQkq - 0 16")
                .unwrap();

        let score = eval.evaluate(&pos);

        println!("score : {}", score);

        assert!(score > 0);
    }

    //rnb1k2r/pp3pbp/1N1ppnp1/2pP4/4P3/3Q1N2/PPP2PPP/R1B1KB1R b KQkq - 0 8
    #[test]
    fn eval_random_pos_4() {
        let model_path = "src/trainer_rust/models/mlp_checkpoint_3.json";

        let mut eval = NeuralEval::<NdArray>::load(model_path);

        let pos = Position::from_fen(
            "rnb1k2r/pp3pbp/1N1ppnp1/2pP4/4P3/3Q1N2/PPP2PPP/R1B1KB1R b KQkq - 0 8",
        )
        .unwrap();

        let score = eval.evaluate(&pos);

        println!("score : {}", score);

        assert!(score > 0);
    }
}
