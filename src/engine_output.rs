use crate::engine_eval::EngineEval;
use std::fmt;

/// Represents the total output from the engine in regards to one specific position.
/// Contains the engine's score evaluation of the position as well as its
/// determined best move.
#[derive(Debug)]
pub struct EngineOutput {
    eval: EngineEval,
    best_move: String,
    pondered_move: Option<String>,
    depth: u32,
}

impl EngineOutput {

    #[must_use]
    pub fn new(eval: EngineEval, best_move: String, pondered_move: Option<String>, depth: u32) -> Self {
        Self { eval, best_move, pondered_move, depth }
    }

    /// Returns [`EngineEval`], a struct representing the engine's
    /// evaluation of the position.
    #[must_use]
    pub fn eval(&self) -> EngineEval {
        self.eval
    }

    /// Returns a string descriptor of the engine's outputted best move.
    /// Given in long UCI algebraic notation (e.g., `"e2e4"`.)
    #[must_use]
    pub fn best_move(&self) -> &String {
        &self.best_move
    }

    /// Returns a string descriptor of the move that the engine had
    /// pondered most. Given in long UCI algebraic notation (e.g., `"e7e5"`.)
    /// 
    /// May be [`None`], if the engine did not output a pondered move.
    #[must_use]
    pub fn pondered_move(&self) -> &Option<String> {
        &self.pondered_move
    }

    /// Returns the depth that the engine had gone to when calculating and upon
    /// returning this output.
    #[must_use]
    pub fn depth(&self) -> u32 {
        self.depth
    }
}
impl fmt::Display for EngineOutput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.eval.to_string() + "_" + &self.best_move.to_string())
    }
}