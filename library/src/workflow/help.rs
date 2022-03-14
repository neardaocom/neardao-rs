use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
};

use crate::{ActionId, ActivityId, ValidatorId};

/// Helper struct to explain Template in human-readable way.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct TemplateHelp {
    description: String,
    conditions: Vec<ConditionHelp>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct ConditionHelp {
    description: String,
    cond_type: Vec<ConditionType>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum ConditionType {
    Transition(ActivityId, ActivityId),
    ExecuteAction(ActivityId, ActionId),
    InputValidator(ActivityId, ActionId, ValidatorId),
}
