use near_sdk::AccountId;
use types::Value;
use workflow::{template::Template, types::ObjectMetadata};

pub mod constants;
pub mod error;
pub mod functions;
pub mod interpreter;
pub mod locking;
pub mod storage;
pub mod tick;
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
pub type ProviderTemplateData = (Template, Vec<FnCallId>, Vec<Vec<ObjectMetadata>>);

/// Timestamp in seconds.
pub type TimestampSec = u64;

/// Flatten object's values type for action input.
pub type ObjectValues = Vec<Vec<Value>>;

/// Version string.
pub type Version = String;

#[macro_export]
macro_rules! derive_into_versioned {
    ($from:ident, $for:ident, $version:ident) => {
        impl From<$from> for $for {
            fn from(input: $from) -> Self {
                $for::$version(input)
            }
        }
    };
}

#[macro_export]
macro_rules! derive_from_versioned {
    ($from:ident, $for:ident, $version:ident) => {
        impl From<$from> for $for {
            fn from(input: $from) -> Self {
                match input {
                    $from::$version(c) => c,
                    _ => unreachable!(),
                }
            }
        }
    };
}
