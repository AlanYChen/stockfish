use std::fmt;
use serde::{Serialize, Serializer};

/// The category of evaluation returned by stockfish. Either `Centipawns` or `Mate`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EvalType {Centipawn, Mate}

impl EvalType {
    pub fn from_descriptor(str: &str) -> EvalType {
        match str {
            "cp" => EvalType::Centipawn,
            "mate" => EvalType::Mate,
            _ => panic!("Unable to create eval type")
        }
    }
}
impl fmt::Display for EvalType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            EvalType::Centipawn => "cp",
            EvalType::Mate => "mate",
        })
    }
}
impl Serialize for EvalType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_str(&self.to_string())
    }
}

/// Stores two bits of info: the type of the evaluation (whether it's centipawns or mate in #)
/// and the numerical value of the evaluation (5 centipawns? Mate in 4?)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EngineEval {
    eval_type: EvalType,
    value: i32,
}

impl EngineEval {
    pub fn new(eval_type: EvalType, value: i32) -> Self {
        Self { eval_type, value }
    }
    pub fn eval_type(&self) -> EvalType {
        self.eval_type
    }
    pub fn value(&self) -> i32 {
        self.value
    }
}
impl fmt::Display for EngineEval {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = self.eval_type().to_string() + " " + &self.value().to_string();
        write!(f, "{}", str)
    }
}
impl Serialize for EngineEval {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_str(&self.to_string())
    }
}