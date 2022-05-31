use near_sdk::serde_json::Error;
use thiserror::Error;

use crate::interpreter::error::EvalError;

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

#[derive(Error, Debug)]
pub enum SourceError {
    #[error("invalid source variant: `{0}`")]
    InvalidSourceVariant(String),
    #[error("tpl value missing for key: `{0}`")]
    TplValueMissing(String),
    #[error("tpl setitngs value missing for key: `{0}`")]
    TplSettingValueMissing(String),
    #[error("propose value missing for key: `{0}`")]
    ProposeValueMissing(String),
    #[error("activity value missing for key: `{0}`")]
    ActivityValueMissing(String),
    #[error("action value missing for key: `{0}`")]
    ActionValueMissing(String),
    #[error("global storage value missing for key: `{0}`")]
    GlobalStorageValueMissing(String),
    #[error("storage value missing for key: `{0}`")]
    StorageValueMissing(String),
    #[error("runtime value missing for key: `{0}`")]
    RuntimeValueMissing(u8),
}

#[derive(Error, Debug)]
pub enum BindingError {
    #[error("collecion prefix: `{0}` is missing")]
    CollectionPrefixMissing(u8),
}

#[derive(Error, Debug)]
pub enum PostprocessingError {
    #[error("storage not provided")]
    StorageMissing,
    #[error("invalid instruction variant")]
    NonBindedInstruction,
    #[error("failed to deserialize into expected type: `{0}`")]
    Deserialize(#[from] Error),
    #[error("unsupported function call result type: `{0}`")]
    UnsupportedFnCallResult(String),
}

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("invalid validator definition")]
    InvalidDefinition,
    #[error("missing key prefix at pos: `{0}`")]
    MissingKeyPrefix(u8),
}

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("missing input for key: `{0}`")]
    MissingInputKey(String),
    #[error("source required input which was not provided")]
    InputNotProvided,
    #[error("cast failed: `{0}`")]
    Cast(#[from] CastError),
    #[error("binding failed: `{0}`")]
    Binding(#[from] BindingError),
    #[error("validation failed: `{0}`")]
    Validation(#[from] ValidationError),
    #[error("fetch from source failed: `{0}`")]
    Source(#[from] SourceError),
    #[error("expression id `{0}` is missing")]
    MissingExpression(u8),
    #[error("evaluation exprsesion failed: `{0}`")]
    Eval(#[from] EvalError),
    #[error("postprocessing failed: `{0}`")]
    Postprocessing(#[from] PostprocessingError),
    #[error("invalid metadata: `{0}`")]
    InvalidMetadata(String),
}
