use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    json_types::U128,
    serde::{Deserialize, Serialize},
};

use crate::types::source::SourceDataVariant;

use super::{
    activity::TransitionLimit,
    types::{ActivityRight, VoteScenario},
};
// TODO: Remove all Debug in production!
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct TemplateSettings {
    pub allowed_proposers: Vec<ActivityRight>,
    pub allowed_voters: ActivityRight,
    /// TODO: Fix. Currently requires pading one vec![] coz Init activity.
    pub activity_rights: Vec<Vec<ActivityRight>>,
    pub transition_limits: Vec<Vec<TransitionLimit>>,
    pub scenario: VoteScenario,
    pub duration: u32,
    pub quorum: u8,
    pub approve_threshold: u8,
    pub spam_threshold: u8,
    pub vote_only_once: bool,
    pub deposit_propose: Option<U128>,
    pub deposit_vote: Option<U128>,
    /// Percents of proposal deposit to be returned.
    pub deposit_propose_return: u8,
    pub constants: Option<SourceDataVariant>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct ProposeSettings {
    /// Top level binds. Shared across all activities.
    pub global: Option<SourceDataVariant>,
    /// Bind per activity. Init activity's binds must be 0th.
    pub binds: Vec<Option<ActivityBind>>,
    /// Storage key under which is the workflow data storage created.
    pub storage_key: Option<String>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct ActivityBind {
    /// Binds shared for all actions.
    pub shared: Option<SourceDataVariant>,
    /// Bind per activity actions.
    pub values: Vec<Option<SourceDataVariant>>,
}
