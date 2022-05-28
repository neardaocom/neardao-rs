use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
};

use crate::{interpreter::expression::EExpr, types::source::SourceDataVariant, Version};

use super::activity::{Activity, Transition};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Template {
    pub code: String,
    pub version: Version,
    /// Workflow can be auto-executed.
    pub auto_exec: bool,
    pub need_storage: bool,
    /// Keys under which its possible to store amounts received by other token contracts.
    pub receiver_storage_keys: Vec<ReceiverKeys>,
    /// First activity is init activity as workflow might diverge from init.
    pub activities: Vec<Activity>,
    /// Expressions shared for all template entities.
    pub expressions: Vec<EExpr>,
    /// Index of transition is id of activity from.
    pub transitions: Vec<Vec<Transition>>,
    pub constants: SourceDataVariant,
    /// Ids of activities which make possible to finish workflow when their are successfully executed.
    pub end: Vec<u8>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct ReceiverKeys {
    pub id: String,
    pub sender_id: String,
    pub token_id: String,
    pub amount: String,
}

// TODO: Implement.
/// Metadata about template.
/// Used to validate ProposeSettings and TemplateSettings.
pub struct TemplateMetadata {}
