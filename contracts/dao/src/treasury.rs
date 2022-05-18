use std::fmt::Display;

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

impl TreasuryPartition {
    pub fn assets(&self) -> &[PartitionAsset] {
        self.assets.as_slice()
    }
    /// Add new asset and returns true if succesfully added.
    pub fn add_asset(&mut self, asset: PartitionAsset) -> bool {
        if self.find_asset_pos(&asset.asset_id()).is_none() {
            self.assets.push(asset);
            true
        } else {
            false
        }
    }
    /// Remove asset if exists.
    pub fn remove_asset(&mut self, asset_id: Asset) -> Option<PartitionAsset> {
        if let Some(pos) = self.find_asset_pos(&asset_id) {
            Some(self.assets.swap_remove(pos))
        } else {
            None
        }
    }
    /// Return reference to the asset if exists.
    pub fn asset(&self, asset_id: &Asset) -> Option<&PartitionAsset> {
        if let Some(pos) = self.find_asset_pos(asset_id) {
            self.assets.get(pos)
        } else {
            None
        }
    }
    /// Add amount to the asset and returns new amount.
    pub fn add_amount(&mut self, asset_id: &Asset, amount: u128) -> u128 {
        if let Some(pos) = self.find_asset_pos(&asset_id) {
            let asset = &mut self.assets[pos];
            asset.add_amount(amount);
            asset.amount
        } else {
            0
        }
    }
    /// Remove max possible `multiple` amount up to `amount` of the asset.
    /// If multiple is 0, up to `amount` is removed.
    /// Return actually removed amount.
    pub fn remove_amount(&mut self, asset_id: &Asset, multiple: u128, max_amount: u128) -> u128 {
        if let Some(pos) = self.find_asset_pos(&asset_id) {
            let asset = &mut self.assets[pos];
            if multiple > 0 {
                let count = std::cmp::min(asset.available_amount(), max_amount) / multiple;
                asset.remove_amount(count * multiple)
            } else {
                asset.remove_amount(max_amount)
            }
        } else {
            0
        }
    }
    /// Unlock all assets with lock.
    pub fn unlock_all(&mut self, current_timestamp: TimestampSec) {
        for asset in self.assets.iter_mut() {
            asset.unlock(current_timestamp);
        }
    }
    fn find_asset_pos(&self, asset_id: &Asset) -> Option<usize> {
        self.assets.iter().position(|el| el.asset_id == *asset_id)
    }
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
#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PartitionAsset {
    asset_id: Asset,
    /// Available amount of the asset with decimal zeroes.
    amount: u128,
    lock: Option<UnlockingDB>,
}

impl PartitionAsset {
    /// Create new self.
    /// Available amount is sum of `amount` and result of immediately called unlock on `lock`.
    /// `amount` is supposed to be integer amount of the asset.
    /// Eg. amount = 2 for asset_id Asset::Near is actually stored as 2 * 10^24.
    pub fn new(
        asset_id: Asset,
        amount: u128,
        lock: Option<UnlockingDB>,
        current_timestamp: TimestampSec,
    ) -> Self {
        let (amount, lock) = if let Some(mut lock) = lock {
            lock.unlock(current_timestamp);
            (
                amount * 10u128.pow(asset_id.decimals() as u32)
                    + lock.available() as u128 * 10u128.pow(asset_id.decimals() as u32),
                Some(lock),
            )
        } else {
            (amount * 10u128.pow(asset_id.decimals() as u32), lock)
        };
        Self {
            asset_id,
            amount,
            lock,
        }
    }
    /// Add amount.
    pub fn add_amount(&mut self, amount: u128) {
        self.amount += amount;
    }
    /// Remove amount up to `amount` and returns actually removed amount.
    pub fn remove_amount(&mut self, amount: u128) -> u128 {
        let remove_amount = std::cmp::min(self.amount, amount);
        self.amount -= remove_amount;
        remove_amount
    }
    /// Unlock all possible tokens and returns new amount.
    pub fn unlock(&mut self, current_timestamp: TimestampSec) -> u128 {
        if let Some(lock) = &mut self.lock {
            let unlocked = lock.unlock(current_timestamp) as u128
                * 10u128.pow(self.asset_id.decimals() as u32);
            self.amount += unlocked;
            self.amount
        } else {
            self.amount
        }
    }
    pub fn asset_id(&self) -> &Asset {
        &self.asset_id
    }
    pub fn available_amount(&self) -> u128 {
        self.amount
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

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, Eq, PartialOrd, Ord)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum Asset {
    Near,
    FT(AssetFT),
    NFT(AssetNFT),
}

impl Display for Asset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Asset::Near => write!(f, "near"),
            Asset::FT(a) => write!(f, "ft:{}", a.account_id),
            Asset::NFT(a) => write!(f, "nft:{};token_id:{}", a.account_id, a.token_id),
        }
    }
}
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, Eq, PartialOrd, Ord)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub struct AssetFT {
    pub account_id: AccountId,
    pub decimals: u8,
}
impl AssetFT {
    pub fn new(account_id: AccountId, decimals: u8) -> Self {
        Self {
            account_id,
            decimals,
        }
    }
}

impl PartialEq for AssetFT {
    fn eq(&self, other: &Self) -> bool {
        self.account_id == other.account_id
    }
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, Eq, PartialOrd, Ord)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub struct AssetNFT {
    pub account_id: AccountId,
    pub token_id: TokenId,
    pub approval_id: ApprovalId,
}

impl PartialEq for AssetNFT {
    fn eq(&self, other: &Self) -> bool {
        self.account_id == other.account_id && self.token_id == other.token_id
    }
}

impl AssetNFT {
    pub fn new(account_id: AccountId, token_id: TokenId, approval_id: ApprovalId) -> Self {
        Self {
            account_id,
            token_id,
            approval_id,
        }
    }
}

impl Asset {
    pub fn new_near() -> Self {
        Self::Near
    }
    pub fn new_ft(account_id: AccountId, decimals: u8) -> Self {
        Self::FT(AssetFT::new(account_id, decimals))
    }
    pub fn new_nft(account_id: AccountId, token_id: String, approval_id: Option<u64>) -> Self {
        Self::NFT(AssetNFT::new(account_id, token_id, approval_id))
    }
    pub fn decimals(&self) -> u8 {
        match &self {
            Self::Near => 24,
            Self::FT(a) => a.decimals,
            _ => 0,
        }
    }
}

impl PartialEq for Asset {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::FT(l), Self::FT(r)) => l == r,
            (Self::NFT(l), Self::NFT(r)) => l == r,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

#[near_bindgen]
impl Contract {
    pub fn unlock_partition_assets(&mut self, id: u16) {
        let mut partition: TreasuryPartition = self
            .treasury_partition
            .get(&id)
            .expect("partition not found")
            .into();
        let current_timestamp = current_timestamp_sec();
        partition.unlock_all(current_timestamp);
        self.treasury_partition.insert(&id, &partition.into());
    }
}

impl Contract {
    pub fn add_partition(&mut self, partition: TreasuryPartition) -> u16 {
        self.partition_last_id += 1;
        self.treasury_partition
            .insert(&self.partition_last_id, &partition.into());
        self.partition_last_id
    }
    pub fn remove_partition(&mut self, partition_id: u16) -> Option<VersionedTreasuryPartition> {
        self.treasury_partition.remove(&partition_id)
    }
}
