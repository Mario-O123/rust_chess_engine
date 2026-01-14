use crate::position::Position;
use super::Evaluator;

pub struct ClassicalEval;

impl ClassicalEval {
    pub fn new() -> Self {Self}
}

impl Evaluator for ClassicalEval {
    //dummy evaluate
    fn evaluate(&mut self, pos: &Position) -> i32 {
        0
    }
}

