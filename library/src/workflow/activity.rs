use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
};

use crate::ActivityId;

use super::{
    action::{ActionData, TemplateAction},
    expression::Expression,
    postprocessing::Postprocessing,
    types::DaoActionIdent,
};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
/// Wrapper around `TemplateActivity`.
/// Init variant is always as first activity in WF.
/// Dao/FnCall activities are basically sync/async activites.
pub enum Activity {
    Init,
    Activity(TemplateActivity),
}

impl Activity {
    pub fn is_dao_activity(&self) -> bool {
        match self {
            Activity::Init => false,
            Activity::Activity(a) => a.is_dao_activity,
        }
    }
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

/// Defines activity relation to workflow finish.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Clone, Copy)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum Terminality {
    /// Is not "end" activity.
    NonTerminal = 0,
    /// Can be closed by user.
    User = 1,
    /// Can be closed by anyone.
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
    pub automatic: bool,
    /// Workflow can be autoclosed when this was successful.
    pub terminal: Terminality,
    pub postprocessing: Option<Postprocessing>,
    pub is_dao_activity: bool,
}

impl TemplateActivity {
    pub fn get_dao_action_ident(&self, id: u8) -> Option<DaoActionIdent> {
        match self.actions.get(id as usize) {
            Some(action) => match &action.action_data {
                ActionData::Action(a) => Some(a.name),
                _ => None,
            },
            _ => None,
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Transition {
    pub activity_id: ActivityId,
    pub cond: Option<Expression>,
    pub time_from_cond: Option<Expression>,
    pub time_to_cond: Option<Expression>,
}
