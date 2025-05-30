//! `stockfish` is a lightweight wrapper library for the Stockfish chess engine.
//! 
//! - **Creation & Setup** — Pass the path to the binary executable to [`Stockfish::new`],
//! and then call [`Stockfish::setup_for_new_game`] to ensure that it is ready for further
//! commands.
//! - **Position** — Use methods like [`Stockfish::set_fen_position`] and 
//! [`Stockfish::play_moves`] to configure the chess position that Stockfish is working with.
//! - **Compute** — Using methods such as [`Stockfish::go`] or [`Stockfish::go_for`], 
//! prompt Stockfish to start calculating given the current chess position.
//! - **Output** — Accessory types have been included ([`EngineEval`], [`EvalType`], [`EngineOutput`])
//! to structure the output from Stockfish after it has concluded its calculations.

mod stockfish;

mod engine_eval;
mod engine_output;

pub use crate::stockfish::Stockfish;
pub use crate::engine_eval::{EngineEval, EvalType};
pub use crate::engine_output::EngineOutput;