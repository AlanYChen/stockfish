use crate::engine_eval::EngineEval;
use serde::{Serialize, Serializer};
use std::fmt;

/// Represents the total output from the engine in regards to one specific position.
/// Contains the engine's score evaluation of the position as well as its
/// determined best move.
#[derive(Debug)]
pub struct EngineOutput {
    eval: EngineEval,
    best_move: String,
}

impl EngineOutput {
    pub fn new(eval: EngineEval, best_move: String) -> Self {
        Self { eval, best_move }
    }

    #[must_use]
    pub fn eval(&self) -> EngineEval {
        self.eval
    }

    #[must_use]
    pub fn best_move(&self) -> &String {
        &self.best_move
    }
}
impl fmt::Display for EngineOutput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.eval.to_string() + "_" + &self.best_move.to_string())
    }
}
impl Serialize for EngineOutput {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_str(&self.to_string())
    }
}