use library::{
    locking::{UnlockingDB, UnlockingInput},
    workflow::{settings::TemplateSettings, template::Template, types::ObjectMetadata},
};
use serde::{Deserialize, Serialize};
use workspaces::AccountId;

use crate::utils::{FnCallId, MethodName};

use super::{group::GroupInput, reward::Asset};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct DaoInit {
    pub total_supply: u32,
    pub decimals: u8,
    pub settings: Settings,
    pub groups: Vec<GroupInput>,
    pub tags: Vec<u16>,
    pub standard_function_calls: Vec<MethodName>,
    pub standard_function_call_metadata: Vec<Vec<ObjectMetadata>>,
    pub function_calls: Vec<FnCallId>,
    pub function_call_metadata: Vec<Vec<ObjectMetadata>>,
    pub workflow_templates: Vec<Template>,
    pub workflow_template_settings: Vec<Vec<TemplateSettings>>,
    pub treasury_partitions: Vec<TreasuryPartitionInput>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Settings {
    pub name: String,
    pub purpose: String,
    pub tags: Vec<u16>,
    pub dao_admin_account_id: AccountId,
    pub dao_admin_rights: Vec<String>, //TODO should be rights
    pub workflow_provider: AccountId,
    pub resource_provider: AccountId,
    pub scheduler: AccountId,
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

impl TryFrom<TreasuryPartitionInput> for TreasuryPartition {
    type Error = String;
    fn try_from(v: TreasuryPartitionInput) -> Result<Self, Self::Error> {
        let mut assets = Vec::with_capacity(v.assets.len());
        for asset in v.assets {
            assets.push(PartitionAsset::try_from(asset)?);
        }
        Ok(Self {
            name: v.name,
            assets,
        })
    }
}

impl TryFrom<PartitionAssetInput> for PartitionAsset {
    type Error = String;
    fn try_from(v: PartitionAssetInput) -> Result<Self, Self::Error> {
        let unlocking_db = UnlockingDB::try_from(v.unlocking)?;
        let amount = unlocking_db.available() as u128 * 10u128.pow(v.asset_id.decimals() as u32);
        let lock = if unlocking_db.total_locked() > 0 {
            Some(unlocking_db)
        } else {
            None
        };
        Ok(Self {
            asset_id: v.asset_id,
            amount,
            lock,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct PartitionAsset {
    asset_id: Asset,
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
