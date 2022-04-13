use std::error::Error;
use std::fmt::{self, Display};

use crate::interpreter::error::EvalError;

#[derive(Debug)]
pub enum TypeError {
    Conversion,
}

impl Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeError::Conversion => write!(f, "Conversion failed"),
        }
    }
}

impl Error for TypeError {}

#[derive(Debug)]
pub enum SourceError {
    InvalidSourceVariant,
    /// Source is not available but workflow requires it.
    SourceMissing,
    /// Case when id pos from ArgSrc enum is not in the source.
    InvalidArgId,
}

impl Display for SourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SourceError::InvalidSourceVariant => write!(f, "Invalid source variant."),
            SourceError::SourceMissing => write!(f, "Missing source."),
            SourceError::InvalidArgId => write!(f, "Argument ID is invalid."),
        }
    }
}

impl Error for SourceError {}

#[derive(Debug)]
/// Unites all underlying error types.
pub enum ProcessingError {
    Conversion,
    Source(SourceError),
    Eval(EvalError),
    UserInput(u8),
    Unreachable,
}

impl Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::Source(e) => {
                write!(f, "Fetching value from source failed: {}", e)
            }
            ProcessingError::Eval(e) => write!(f, "Evaluation failed: {}", e),
            ProcessingError::UserInput(id) => write!(f, "Missing value at pos: {}.", id),
            ProcessingError::Conversion => {
                write!(f, "Converting datatype failed - got invalid datatype.")
            }
            ProcessingError::Unreachable => write!(f, "Processing reached unreachable branch."),
        }
    }
}

impl Error for ProcessingError {}

impl From<SourceError> for ProcessingError {
    fn from(e: SourceError) -> Self {
        Self::Source(e)
    }
}

impl From<EvalError> for ProcessingError {
    fn from(e: EvalError) -> Self {
        Self::Eval(e)
    }
}

impl From<TypeError> for ProcessingError {
    fn from(_e: TypeError) -> Self {
        Self::Conversion
    }
}
