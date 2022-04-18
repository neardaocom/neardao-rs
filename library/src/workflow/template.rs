use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
};

use crate::{types::datatype::Value, Version};

use super::{
    activity::{Activity, TemplateActivity, Transition},
    expression::Expression,
};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Template {
    pub code: String,
    pub version: Version,
    /// Whole workflow can be autoexecuted.
    pub is_simple: bool,
    pub need_storage: bool,
    //TODO instead of null must be "start" activity
    pub activities: Vec<Activity>,
    //pub obj_validators: Vec<Vec<ValidatorType>>,
    pub validator_exprs: Vec<Expression>, //TODO move to matrix, this wont work in all scenarios or use ValidatorId in obj_validators
    pub transitions: Vec<Vec<Transition>>,
    pub expressions: Vec<Expression>, //TODO move to action?
    pub constants: Vec<Value>,
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
