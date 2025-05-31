use std::fmt;

/// The category of evaluation returned by stockfish. Either `Centipawns` or `Mate`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EvalType {Centipawn, Mate}

impl EvalType {

    /// Creates an [`EvalType`] from a string descriptor. The valid descriptors are:
    /// - `"cp"`, which translates to [`EvalType::Centipawn`]
    /// - `"mate"`, which translates to [`EvalType::Mate`]
    /// 
    /// # Panics
    /// 
    /// This function panics when given a string descriptor that doesn't match those listed above.
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

/// Represents the evaluation returned from the engine. Includes [`EvalType`] and a numerical score value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EngineEval {
    eval_type: EvalType,
    value: i32,
}

impl EngineEval {
    pub fn new(eval_type: EvalType, value: i32) -> Self {
        Self { eval_type, value }
    }

    #[must_use]
    pub fn eval_type(&self) -> EvalType {
        self.eval_type
    }

    #[must_use]
    pub fn value(&self) -> i32 {
        self.value
    }
}
impl fmt::Display for EngineEval {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = self.eval_type().to_string() + " " + &self.value().to_string();
        write!(f, "{str}")
    }
}