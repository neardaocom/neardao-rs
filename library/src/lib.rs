use near_sdk::AccountId;
use types::datatype::{Datatype, Value};
use workflow::{template::Template, types::ObjectMetadata};

pub mod functions;
pub mod interpreter;
pub mod storage;
pub mod types;
pub mod workflow;

pub type MethodName = String;
pub type FnCallId = (AccountId, MethodName);
pub type EventCode = String;

pub type ActivityId = u8;
pub type ActionId = u8;
pub type ObjectId = u8;
pub type BindId = u8;
pub type ValidatorId = u8;
pub type ExpressionId = u8;
pub type TransitionId = u8;
pub type FnCallResultDatatype = Option<Datatype>;
pub type ProviderTemplateData = (Template, Vec<FnCallId>, Vec<Vec<ObjectMetadata>>);

/// Flatten object's values type for action input.
pub type ObjectValues = Vec<Vec<Value>>;

/// Version string.
/// Max 16 characters (unchecked atm).
pub type Version = String;
