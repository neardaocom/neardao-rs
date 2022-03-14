use std::error::Error;
use std::fmt::{self, Display};

#[derive(Debug)]
pub enum EvalError {
    InvalidType,
    Unimplemented,
}

impl Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalError::InvalidType => write!(f, "Invalid datatype"),
            EvalError::Unimplemented => write!(f, "Unimplemented"),
        }
    }
}

impl Error for EvalError {}
