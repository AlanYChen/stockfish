//! `stockfish` is a small library for creating and interacting with a running
//! Stockfish process.
//! 
//! It includes some accessory types (`EngineEval`, `EvalType`, `EngineOutput`) for
//! working with output received from the Stockfish process.

mod stockfish;

mod engine_eval;
mod engine_output;

pub use crate::stockfish::Stockfish;
pub use crate::engine_eval::{EngineEval, EvalType};
pub use crate::engine_output::EngineOutput;