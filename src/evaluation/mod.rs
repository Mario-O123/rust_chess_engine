//! position evaluation:
//! this module defines a small abstraction for evaluating a [`Position`]
//! and provides concrete backends:
//!
//! -[`classical`] -a hand crafted evaluator
//! -[`neural`] -a optional neural-network evaluator (enabled with nn-feature)
//!
//! most callers interact with [`EvalEngine`] which wraps the selected backend behind a single type

use crate::position::Position;

pub mod classical;
pub use classical::ClassicalEval;
#[cfg(feature = "nn")]
pub mod neural;

///common interface for evaluating a [`Position`]
pub trait Evaluator {
    fn evaluate(&mut self, pos: &Position) -> i32;
}

///concrete evaluation engine, this enum wraps the available backends and offers a uniform
///[`evaluate`](Self::evaluate) entry point
pub enum EvalEngine {
    Classical(classical::ClassicalEval),
    #[cfg(feature = "nn")]
    Neural(neural::NeuralEval),
}

impl EvalEngine {
    ///construct a classical evaluator
    pub fn classical() -> Self {
        Self::Classical(classical::ClassicalEval::new())
    }

    ///construct a neural evaluator from a model file
    #[cfg(feature = "nn")]
    pub fn neural(path: &str) -> anyhow::Result<Self> {
        Ok(Self::Neural(neural::NeuralEval::load(path)?))
    }

    ///evaluate a position using the configured backend
    pub fn evaluate(&mut self, pos: &Position) -> i32 {
        match self {
            EvalEngine::Classical(e) => e.evaluate(pos),
            #[cfg(feature = "nn")]
            EvalEngine::Neural(e) => e.evaluate(pos),
        }
    }
}
