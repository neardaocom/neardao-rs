use std::error::Error;
use std::fmt::{self, Display};

#[derive(Debug)]
pub enum WfError {
    FatalError(EvalError),
}

impl Display for WfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WfError::FatalError(e) => write!(f, "Evaluation failed: {}", e),
        }
    }
}

impl Error for WfError {}
