use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    json_types::U128,
    serde::{Deserialize, Serialize},
};

use super::{
    activity::TransitionLimit,
    template::SourceDataVariant,
    types::{ActivityRight, VoteScenario},
};
// TODO: Remove all Debug in production!
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct TemplateSettings {
    pub allowed_proposers: Vec<ActivityRight>,
    pub allowed_voters: ActivityRight,
    pub activity_rights: Vec<Vec<ActivityRight>>,
    pub transition_limits: Vec<Vec<TransitionLimit>>,
    pub scenario: VoteScenario,
    pub duration: u32,
    pub quorum: u8,
    pub approve_threshold: u8,
    pub spam_threshold: u8,
    pub vote_only_once: bool,
    /// Deposit required to be able to propose.
    pub deposit_propose: Option<U128>,
    /// Deposit required to be able to vote in the proposal.
    pub deposit_vote: Option<U128>,
    /// Percents of `deposit_propose` to be returned when proposal is Accepted.
    pub deposit_propose_return: u8,
    pub constants: Option<SourceDataVariant>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct ProposeSettings {
    /// Top level binds. Shared across all activities.
    pub constants: Option<SourceDataVariant>,
    /// Constants per activity. Init activity's binds must be 0th and always `None`.
    pub activity_constants: Vec<Option<ActivityBind>>,
    /// Storage key under which is the workflow data storage created.
    pub storage_key: Option<String>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct ActivityBind {
    /// Binds shared for all actions.
    pub constants: Option<SourceDataVariant>,
    /// Bind per activity actions.
    pub actions_constants: Vec<Option<SourceDataVariant>>,
}
