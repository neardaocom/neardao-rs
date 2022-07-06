use thiserror::Error;

#[derive(Error, Debug)]
#[error("failed to cast datatype {from} to {to}")]
pub struct CastError {
    from: String,
    to: String,
}

impl CastError {
    pub fn new(from: &str, to: &str) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
        }
    }
}
