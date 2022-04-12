use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
    AccountId,
};

use crate::{derive_from_versioned, derive_into_versioned, TagId};

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

pub(crate) fn assert_valid_dao_settings(settings: &DaoSettings) {
    assert!(!settings.name.is_empty());
    assert!(!settings.dao_admin_account_id.is_empty()); //TODO switch to valid account_id check in SDK 4.0
    assert!(!settings.workflow_provider.is_empty());
}
