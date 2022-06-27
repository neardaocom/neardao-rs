use library::{
    locking::{UnlockingDB, UnlockingInput},
    workflow::{settings::TemplateSettings, template::Template, types::ObjectMetadata},
};
use serde::{Deserialize, Serialize};
use workspaces::AccountId;

use crate::utils::{FnCallId, MethodName};

use super::{group::GroupInput, reward::Asset, Media};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct DaoInit {
    pub total_supply: u32,
    pub decimals: u8,
    pub settings: DaoSettings,
    pub groups: Vec<GroupInput>,
    pub tags: Vec<u16>,
    pub standard_function_calls: Vec<MethodName>,
    pub standard_function_call_metadata: Vec<Vec<ObjectMetadata>>,
    pub function_calls: Vec<FnCallId>,
    pub function_call_metadata: Vec<Vec<ObjectMetadata>>,
    pub workflow_templates: Vec<Template>,
    pub workflow_template_settings: Vec<Vec<TemplateSettings>>,
    pub treasury_partitions: Vec<TreasuryPartitionInput>,
    pub media: Vec<Media>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct DaoSettings {
    pub name: String,
    pub purpose: String,
    pub tags: Vec<u16>,
    pub dao_admin_account_id: AccountId,
    pub dao_admin_rights: Vec<AdminRight>,
    pub workflow_provider: AccountId,
    pub resource_provider: Option<AccountId>,
    pub scheduler: Option<AccountId>,
    /// Vote token id.
    pub token_id: AccountId,
    /// Staking contract.
    pub staking_id: AccountId,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct TreasuryPartitionInput {
    pub name: String,
    pub assets: Vec<PartitionAssetInput>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct PartitionAssetInput {
    pub asset_id: Asset,
    pub unlocking: UnlockingInput,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct PartitionAsset {
    asset_id: u8,
    decimals: u8,
    /// Available amount of the asset with decimal zeroes.
    amount: u128,
    lock: Option<UnlockingDB>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct TreasuryPartition {
    pub name: String,
    pub assets: Vec<PartitionAsset>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum AdminRight {
    Upgrade,
}
