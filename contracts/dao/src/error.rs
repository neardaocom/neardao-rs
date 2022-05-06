use std::{
    error::Error,
    fmt::{self, Display},
};

use library::types::error::{ProcessingError, TypeError};
use near_sdk::{serde::Serialize, serde_json::Error as SerdeError};

pub const ERR_INVALID_AMOUNT: &str = "Invalid amount";
pub const ERR_NO_ACCESS: &str = "You have no rights for this action";
pub const ERR_GROUP_NOT_FOUND: &str = "Group does not exist";
pub const ERR_GROUP_HAS_NO_LEADER: &str = "Group has no leader";
pub const ERR_UNKNOWN_FNCALL: &str = "Undefined fn call";

pub const ERR_LOCK_AMOUNT_OVERFLOW: &str = "Total FT locked amount exceeded total FT supply";
pub const ERR_DISTRIBUTION_AMOUNT_ABOVE: &str = "Total FT distribution exceeded total FT supply";
pub const ERR_DISTRIBUTION_ACC_EMPTY: &str = "No accounts to distribute to";
pub const ERR_DISTRIBUTION_MIN_VALUE: &str = "Cannot distribute less than 1.0 FT per member";
pub const ERR_DISTRIBUTION_NOT_ENOUGH_FT: &str = "Not enough free FT to do init distribution";

pub const ERR_STORAGE_INVALID_TYPE: &str = "Invalid storage data type";
pub const ERR_STORAGE_BUCKET_EXISTS: &str = "Storage bucket already exists";

pub const ERR_PROMISE_INVALID_VALUE: &str = "Promise returned invalid type";
pub const ERR_PROMISE_FAILED: &str = "Promise failed";
pub const ERR_PROMISE_INVALID_RESULTS_COUNT: &str = "Invalid promise results count";

#[derive(Debug, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub enum ActivityError {
    Error(String),
    FatalError(String),
}

impl ActivityError {
    pub fn is_fatal(&self) -> bool {
        matches!(self, Self::FatalError(_))
    }
}

impl Display for ActivityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActivityError::Error(reason) => write!(
                f,
                "Activity failed. Not all actions were executed. Reason: {}",
                reason
            ),
            ActivityError::FatalError(reason) => write!(f, "Reason: {}", reason),
            _ => unimplemented!(),
        }
    }
}

impl AsRef<str> for ActivityError {
    fn as_ref(&self) -> &str {
        match self {
            Self::Error(s) => s.as_str(),
            Self::FatalError(s) => s.as_str(),
        }
    }
}

impl Error for ActivityError {}

impl From<ActionError> for ActivityError {
    fn from(error: ActionError) -> Self {
        match error {
            ActionError::Validation(_) => ActivityError::Error(error.to_string()),
            _ => ActivityError::FatalError(error.to_string()),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub enum ActionError {
    ActionPostprocessing(u8),
    PromiseFailed(u8, u8),
    NotEnoughDeposit,
    InvalidSource,
    InvalidDataType,
    InvalidWfStructure(String),
    MissingFnCallMetadata(String),
    Binding,
    SerDe,
    Validation(u8),
    Condition(u8),
    InputStructure(u8),
    Other(String),
}

impl Display for ActionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActionError::Validation(action_id) => {
                write!(f, "Validation failed. Action: {}.", action_id)
            }
            ActionError::ActionPostprocessing(id) => {
                write!(f, "Postprocessing failed. Action: {}.", id)
            }
            ActionError::PromiseFailed(activity_id, action_id) => {
                write!(
                    f,
                    "FnCall promise failed. Activity: {}, Action: {}.",
                    activity_id, action_id
                )
            }
            ActionError::Condition(action_id) => {
                write!(f, "Action condition failed. Action: {}.", action_id)
            }
            ActionError::InvalidWfStructure(s) => {
                write!(f, "WF has invalid structure: {}", s)
            }
            ActionError::InputStructure(action_id) => {
                write!(f, "Input has invalid structure. Action: {}", action_id)
            }
            ActionError::Binding => write!(f, "Binding failed."),
            ActionError::SerDe => write!(f, "Ser/Deser failed."),
            ActionError::InvalidDataType => write!(f, "Invalid DataType value."),
            ActionError::Other(reason) => write!(f, "{}", reason),
            ActionError::NotEnoughDeposit => {
                write!(f, "Not enough deposit to execute all Event actions.")
            }
            ActionError::MissingFnCallMetadata(fncall_name) => {
                write!(f, "Metadata for FnCall: {} not found.", fncall_name)
            }
            ActionError::InvalidSource => {
                write!(f, "Source was missing or value pos does not match.")
            }
        }
    }
}

impl AsRef<str> for ActionError {
    fn as_ref(&self) -> &str {
        "Action error occured"
    }
}

impl Error for ActionError {}

impl From<TypeError> for ActionError {
    fn from(_: TypeError) -> Self {
        Self::InvalidDataType
    }
}

impl From<SerdeError> for ActionError {
    fn from(_: SerdeError) -> Self {
        Self::SerDe
    }
}

// TODO: improve
impl From<ProcessingError> for ActionError {
    fn from(error: ProcessingError) -> Self {
        match error {
            ProcessingError::Conversion => Self::InvalidDataType,
            ProcessingError::Source(s) => Self::InvalidWfStructure(format!("{:?}", s)),
            ProcessingError::Eval(e) => Self::InvalidWfStructure(format!("{:?}", e)),
            ProcessingError::UserInput(pos) => Self::InputStructure(pos),
            ProcessingError::Unreachable => Self::InvalidWfStructure("unreachable".into()),
            ProcessingError::InvalidValidatorDefinition => todo!(),
            ProcessingError::MissingExpression => todo!(),
            ProcessingError::InvalidExpressionStructure => todo!(),
        }
    }
}
