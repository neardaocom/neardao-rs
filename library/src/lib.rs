use near_sdk::AccountId;
use types::DataType;

mod data;
mod unit_tests;

pub mod expression;
pub mod storage;
pub mod types;
pub mod utils;
pub mod workflow;

pub type MethodName = String;
pub type FnCallId = (AccountId, MethodName);
pub type ArgValidatorId = u8;
pub type BindId = u8;
pub type Consts = dyn Fn(u8) -> DataType;
pub type EventCode = String;
pub type TransitionId = u8;
pub type ActivityId = u8;
