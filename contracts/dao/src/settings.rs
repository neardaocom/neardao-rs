use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
    AccountId,
};

use crate::{derive_from_versioned, derive_into_versioned, TagId};

#[derive(BorshDeserialize, BorshSerialize)]
pub enum VersionedSettings {
    Current(Settings),
}

// TODO: Resource provider.
// TODO: Tick settings.
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Settings {
    pub name: String,
    pub purpose: String,
    pub tags: Vec<TagId>,
    pub dao_admin_account_id: AccountId,
    pub dao_admin_rights: Vec<String>, //TODO should be rights
    pub workflow_provider: AccountId,
}

derive_from_versioned!(VersionedSettings, Settings);
derive_into_versioned!(Settings, VersionedSettings);

pub(crate) fn assert_valid_dao_settings(settings: &Settings) {
    assert!(!settings.name.is_empty());
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct TokenSettings {
    mint_allowed: bool,
    burning_allowed: bool,
}
