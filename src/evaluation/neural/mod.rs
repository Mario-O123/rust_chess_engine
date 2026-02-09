use super::Evaluator;
use crate::position::Position;

pub struct NeuralEval;

impl NeuralEval {
    pub fn new() -> Self {
        Self
    }

    //dummy load
    pub fn load(path: &str) -> anyhow::Result<Self> {
        Ok(Self)
    }
}

impl Evaluator for NeuralEval {
    //dummy eval
    fn evaluate(&mut self, pos: &Position) -> i32 {
        0
    }
}
