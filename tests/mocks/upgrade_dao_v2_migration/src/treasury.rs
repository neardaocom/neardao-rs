use library::locking::{UnlockingDB, UnlockingInput};
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    near_bindgen,
    serde::{Deserialize, Serialize},
    AccountId,
};

use crate::{
    core::*, derive_from_versioned, derive_into_versioned, internal::utils::current_timestamp_sec,
    ApprovalId, TimestampSec, TokenId,
};

derive_into_versioned!(TreasuryPartition, VersionedTreasuryPartition);
derive_from_versioned!(VersionedTreasuryPartition, TreasuryPartition);

#[derive(BorshDeserialize, BorshSerialize)]
pub enum VersionedTreasuryPartition {
    Current(TreasuryPartition),
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TreasuryPartitionInput {
    pub name: String,
    pub assets: Vec<PartitionAssetInput>,
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PartitionAssetInput {
    pub asset_id: Asset,
    pub unlocking: UnlockingInput,
}

/// Container around unique assets.
#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TreasuryPartition {
    pub name: String,
    pub assets: Vec<PartitionAsset>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PartitionAsset {
    asset_id: Asset,
    /// Available amount of the asset with decimal zeroes.
    amount: u128,
    lock: Option<UnlockingDB>,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum Asset {
    Near,
    Ft(AssetFT),
    Nft(AssetNFT),
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub struct AssetFT {
    pub account_id: AccountId,
    pub decimals: u8,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub struct AssetNFT {
    pub account_id: AccountId,
    pub token_id: TokenId,
    pub approval_id: ApprovalId,
}
