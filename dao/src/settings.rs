use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
    AccountId,
};

use crate::{
    constants::MIN_VOTING_DURATION_SEC, derive_from_versioned, derive_into_versioned, TagId,
};

#[derive(BorshDeserialize, BorshSerialize)]
pub enum VDaoSettings {
    Curr(DaoSettings),
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct DaoSettings {
    pub name: String,
    pub purpose: String,
    pub tags: Vec<TagId>,
    pub dao_admin_account_id: AccountId,
    pub dao_admin_rights: Vec<String>, //TODO should be rights
    pub workflow_provider: AccountId,
}

derive_from_versioned!(VDaoSettings, DaoSettings);
derive_into_versioned!(DaoSettings, VDaoSettings);

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum VoteScenario {
    Democratic,
    TokenWeighted,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub enum VVoteSettings {
    Curr(VoteSettings),
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct VoteSettings {
    pub scenario: VoteScenario,
    pub duration: u32,
    pub quorum: u8,
    pub approve_threshold: u8,
    pub spam_threshold: u8,
}

derive_from_versioned!(VVoteSettings, VoteSettings);
derive_into_versioned!(VoteSettings, VVoteSettings);

pub(crate) fn assert_valid_dao_settings(settings: &DaoSettings) {
    assert!(settings.name.len() > 0);
    assert!(settings.dao_admin_account_id.len() > 0); //TODO switch to valid account_id check in SDK 4.0
    assert!(settings.workflow_provider.len() > 0);
}

pub(crate) fn assert_valid_vote_settings(settings: &Vec<VoteSettings>) {
    assert!(!settings.is_empty()); //At least one vote_settings must be provided
    assert!(settings.iter().all(|e| validate_vote_settings(e)));
}

fn validate_vote_settings(settings: &VoteSettings) -> bool {
    if settings.quorum > 100 || settings.approve_threshold > 100 || settings.spam_threshold > 100 {
        return false;
    }

    // min is 5 minutes
    if settings.duration < MIN_VOTING_DURATION_SEC {
        return false;
    }

    true
}
