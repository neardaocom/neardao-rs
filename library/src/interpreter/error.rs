use thiserror::Error;

use crate::types::error::CastError;

#[derive(Error, Debug)]
pub enum EvalError {
    #[error("Invalid datatype")]
    InvalidDatatype,
    #[error("Operation is not implemented")]
    Unimplemented,
    #[error("Cast error: `{0}`")]
    Cast(#[from] CastError),
    #[error("Expression expects min: `{0}` args")]
    InvalidArgCount(u64),
    #[error("Invalid operands: `{0}`")]
    InvalidOperands(String),
    #[error("Missing arg atpos: `{0}`")]
    MissingArg(u64),
}
