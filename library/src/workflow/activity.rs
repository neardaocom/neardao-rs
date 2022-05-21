use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
};

use crate::ActivityId;

use super::{action::TemplateAction, postprocessing::Postprocessing, types::ValueSrc};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
/// Wrapper around `TemplateActivity`.
/// Init variant is always as first activity in WF.
/// Dao/FnCall activities are basically sync/async activites.
pub enum Activity {
    Init,
    Activity(TemplateActivity),
}

impl Activity {
    pub fn into_activity(self) -> Option<TemplateActivity> {
        match self {
            Activity::Init => None,
            Activity::Activity(a) => Some(a),
        }
    }

    pub fn activity_as_ref(&self) -> Option<&TemplateActivity> {
        match self {
            Activity::Init => None,
            Activity::Activity(ref a) => Some(a),
        }
    }

    pub fn activity_as_mut(&mut self) -> Option<&mut TemplateActivity> {
        match self {
            Activity::Init => None,
            Activity::Activity(a) => Some(a),
        }
    }
}

// TODO: Remove Debug in production.
/// Define activity relation to workflow finish.
#[derive(
    BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Clone, Copy, Debug,
)]
//#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum Terminality {
    /// Not terminal activity.
    NonTerminal = 0,
    /// Terminal activity.
    /// Can be closed only by user.
    User = 1,
    /// Terminal activity.
    /// Can be closed by anyone - usually closed by the executing engine.
    Automatic = 2,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct TemplateActivity {
    pub code: String,
    /// 1..N actions, only gas is the limit!
    pub actions: Vec<TemplateAction>,
    /// Execution can be done by anyone anytime.
    /// This should be true only when no user inputs are required by any of the contained actions.
    pub automatic: bool,
    /// Relation to autoclosing workflow and successful execution of the activity.
    pub terminal: Terminality,
    /// Postprocessing script in case of successfull execution.
    pub postprocessing: Option<Postprocessing>,
    /// Helper flag for executing engine.
    pub is_sync: bool,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Transition {
    pub activity_id: ActivityId,
    pub cond: Option<ValueSrc>,
    pub time_from_cond: Option<ValueSrc>,
    pub time_to_cond: Option<ValueSrc>,
}

// TODO: Remove Debug in production.
/// From activity_id is defined by its position in the hosting container (Vec).
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct TransitionLimit {
    /// Target activity id.
    pub to: u8,
    /// Transition limit.
    pub limit: u16,
}

/// From activity_id is defined by its position in the hosting container (Vec).
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct TransitionCounter {
    /// Target activity id.
    pub to: u8,
    /// Transition counter to activity `to`.
    pub count: u16,
    /// Transition limit.
    pub limit: u16,
}

impl TransitionCounter {
    pub fn is_transition_allowed(&self) -> bool {
        self.count < self.limit
    }
    pub fn inc_count(&mut self) {
        self.count += 1;
    }
}
