use std::error::Error;
use std::fmt::{self, Display};

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
