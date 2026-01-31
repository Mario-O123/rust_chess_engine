use crate::position::Position;

pub mod classical;
pub use classical::ClassicalEval;
#[cfg(feature = "nn")]
pub mod neural;
pub trait Evaluator {
    fn evaluate(&mut self, pos: &Position) -> i32;
}

pub enum EvalEngine {
    Classical(classical::ClassicalEval),
    #[cfg(feature = "nn")]
    Neural(neural::NeuralEval),
}

impl EvalEngine {
    pub fn classical() -> Self {
        Self::Classical(classical::ClassicalEval::new())
    }

    #[cfg(feature = "nn")]
    pub fn neural(path: &str) -> anyhow::Result<Self> {
        Ok(Self::Neural(neural::NeuralEval::load(path)?))
    }

    pub fn evaluate(&mut self, pos: &Position) -> i32 {
        match self {
            EvalEngine::Classical(e) => e.evaluate(pos),
            #[cfg(feature = "nn")]
            EvalEngine::Neural(e) => e.evaluate(pos),
        }
    }
}
