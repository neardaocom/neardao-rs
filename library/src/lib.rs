use near_sdk::AccountId;

pub mod expression;
pub mod storage;
pub mod types;
pub mod utils;
pub mod workflow;

pub type MethodName = String;
pub type FnCallId = (AccountId, MethodName);
pub type ArgValidatorId = u8;
pub type BindId = u8;
