use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
};

use crate::{interpreter::expression::EExpr, types::source::SourceDataVariant, Version};

use super::activity::{Activity, TemplateActivity, Transition};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Template {
    pub code: String,
    pub version: Version,
    /// Workflow can be auto-executed.
    pub auto_exec: bool,
    pub need_storage: bool,
    /// First activity is init activity as workflow might diverge from init.
    pub activities: Vec<Activity>,
    /// Expressions shared for all template entities.
    pub expressions: Vec<EExpr>,
    /// Index of transition is id of activity from.
    pub transitions: Vec<Vec<Transition>>,
    // TODO figure out structure.
    pub constants: SourceDataVariant,
    /// Ids of activities which make possible to finish workflow when their are successfully executed.
    pub end: Vec<u8>,
}

impl Template {
    /// Returns reference to inner activity. Init activity is considered as no activity so id 0 always returns `None`.
    pub fn get_activity_as_ref(&self, id: u8) -> Option<&TemplateActivity> {
        match self.activities.get(id as usize) {
            Some(activity) => activity.activity_as_ref(),
            None => None,
        }
    }
}

// TODO: Implement.
/// Metadata about template.
/// Used to validate ProposeSettings and TemplateSettings.
pub struct TemplateMetadata {}
