use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    json_types::U128,
    serde::{Deserialize, Serialize},
};

use crate::{types::DataType, TransitionLimit};

use super::types::{ActivityRight, VoteScenario};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
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
    pub deposit_propose: Option<U128>,
    pub deposit_vote: Option<U128>,
    pub deposit_propose_return: u8, // how many percent of propose deposit to return, TODO implement
    pub constants: Vec<DataType>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct ProposeSettings {
    pub binds: Vec<Option<ActivityBind>>,
    pub storage_key: Option<String>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct ActivityBind {
    pub shared: Vec<DataType>,
    pub values: Vec<Vec<DataType>>,
}
