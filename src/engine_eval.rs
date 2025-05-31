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

/// Represents the evaluation returned from the engine. Includes an [`EvalType`] and a numerical score value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EngineEval {
    eval_type: EvalType,
    value: i32,
    depth: u32,
}

impl EngineEval {
    pub fn new(eval_type: EvalType, value: i32, depth: u32) -> Self {
        Self { eval_type, value, depth }
    }

    /// Returns an [`EvalType`] representing what type of evaluation was returned
    /// from the engine: whether the evaluation is expressed in centipawns or as
    /// mate in a certain number of moves.
    #[must_use]
    pub fn eval_type(&self) -> EvalType {
        self.eval_type
    }

    /// Returns a number representing the numerical value associated with the
    /// evaluation returned from the engine; this number is expressed in centipawns
    /// or the number of moves in which mate may be forced (depending on [`EvalType`].)
    #[must_use]
    pub fn value(&self) -> i32 {
        self.value
    }

    /// Returns the depth that the engine had gone to when calculating and upon
    /// returning this output.
    pub fn depth(&self) -> u32 {
        self.depth
    }
}
impl fmt::Display for EngineEval {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = self.eval_type().to_string() + " " + &self.value().to_string();
        write!(f, "{str}")
    }
}