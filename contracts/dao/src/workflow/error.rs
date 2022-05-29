use library::types::error::{CastError, ProcessingError};
use near_sdk::{serde::Serialize, ParseAccountIdError};
use thiserror::Error;

#[derive(Error, Debug, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub enum ActivityError {
    #[error("user input error: `{0}`")]
    Input(String),
    #[error("fatal error happened: `{0}`")]
    Fatal(String),
}

impl From<ActionError> for ActivityError {
    fn from(a: ActionError) -> Self {
        match a {
            ActionError::WorkflowInternal(e) => {
                if let ProcessingError::MissingInputKey(key) = e {
                    Self::Input(format!("missing user input key: {}", key))
                } else {
                    Self::Fatal(e.to_string())
                }
            }
            ActionError::NotEnoughDeposit => Self::Input(a.to_string()),
            ActionError::Validation => Self::Input("validation failed".into()),
            ActionError::Condition(_) => Self::Input(a.to_string()),
            ActionError::ParseAccountId(from_user) => {
                if from_user {
                    Self::Input("invalid account id string".into())
                } else {
                    Self::Fatal(a.to_string())
                }
            }
            _ => Self::Fatal(a.to_string()),
        }
    }
}

impl ActivityError {
    pub fn is_fatal(&self) -> bool {
        matches!(self, Self::Fatal(_))
    }
}

#[derive(Error, Debug)]
pub enum ActionError {
    #[error("not enough deposit")]
    NotEnoughDeposit,
    #[error("exec condition id: `{0}` is invalid")]
    Condition(u8),
    #[error("user input is not valid")]
    Validation,
    #[error("workflow structure is invalid: `{0}`")]
    InvalidWfStructure(String),
    #[error("failed to cast value: `{0}`")]
    Cast(#[from] CastError),
    #[error("failed to parse account id, value is from user input: `{0}`")]
    ParseAccountId(bool),
    #[error("internal workflow error: `{0}`")]
    WorkflowInternal(#[from] ProcessingError),
    #[error("failed to deserialize dao object: `{0}`")]
    DeserializeDaoObject(#[from] DeserializeError),
    #[error("internal dao action error: `{0}`")]
    DaoActionInternal(#[from] InternalDaoActionError),
}

#[derive(Error, Debug)]
pub enum DeserializeError {
    #[error("invalid datatype: `{0}`")]
    Cast(#[from] CastError),
    #[error("missing key: `{0}`")]
    MissingInputKey(String),
    #[error("failed to convert input to: `{0}`")]
    Conversion(String),
    #[error("failed to parse account id: `{0}`")]
    AccountIdParse(#[from] ParseAccountIdError),
}

#[derive(Error, Debug)]
#[error("internal action error: `{0}`")]
pub struct InternalDaoActionError(pub String);
