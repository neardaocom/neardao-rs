use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
    AccountId,
};

use crate::{contract::Contract, derive_from_versioned, derive_into_versioned, TagId};

#[derive(BorshDeserialize, BorshSerialize)]
pub enum VersionedSettings {
    Current(Settings),
}
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Settings {
    pub name: String,
    pub purpose: String,
    pub tags: Vec<TagId>,
    pub dao_admin_account_id: AccountId,
    pub dao_admin_rights: Vec<AdminRight>, // TODO: Fix - should be rights.
    pub workflow_provider: AccountId,
    pub resource_provider: Option<AccountId>,
    pub scheduler: Option<AccountId>,
    /// Vote token id.
    pub token_id: AccountId,
    /// Staking contract.
    pub staking_id: AccountId,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum AdminRight {
    Upgrade,
}

derive_from_versioned!(VersionedSettings, Settings);
derive_into_versioned!(Settings, VersionedSettings);

pub(crate) fn assert_valid_dao_settings(settings: &Settings) {
    assert!(!settings.name.is_empty());
}

impl Contract {
    pub fn settings_update(&mut self, settings: Settings) {
        assert_valid_dao_settings(&settings);
        self.settings.replace(&settings.into());
    }
}
