use library::locking::Lock;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
    AccountId,
};

use crate::{core::Contract, ApprovalId, TimestampSec, TokenId};

/// Container around unique assets.
#[derive(BorshDeserialize, BorshSerialize)]
pub struct TreasuryPartition {
    pub assets: Vec<PartitionAsset>,
}

impl Default for TreasuryPartition {
    fn default() -> Self {
        Self { assets: vec![] }
    }
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
    pub fn asset(&self, asset_id: Asset) -> Option<&PartitionAsset> {
        if let Some(pos) = self.find_asset_pos(&asset_id) {
            self.assets.get(pos)
        } else {
            None
        }
    }
    /// Add amount to the asset and returns new amount.
    pub fn add_amount(&mut self, asset_id: &Asset, amount: u128) -> u128 {
        if let Some(pos) = self.find_asset_pos(&asset_id) {
            let asset = self.assets.get_mut(pos).unwrap();
            asset.add_amount(amount);
            asset.amount
        } else {
            0
        }
    }
    /// Remove max possible amount up to `amount` of the asset.
    /// Return actually removed amount.
    pub fn remove_amount(&mut self, asset_id: &Asset, amount: u128) -> u128 {
        if let Some(pos) = self.find_asset_pos(&asset_id) {
            self.assets.get_mut(pos).unwrap().remove_amount(amount)
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
#[derive(BorshDeserialize, BorshSerialize)]
pub struct PartitionAsset {
    asset_id: Asset,
    /// Available amount of the asset with decimal zeroes.
    amount: u128,
    lock: Option<Lock>,
}

impl PartitionAsset {
    /// Creates new self.
    /// Available amount is sum of `amount` and result of immediately called unlock `lock`.
    pub fn new(
        asset_id: Asset,
        amount: u128,
        lock: Option<Lock>,
        current_timestamp: TimestampSec,
    ) -> Self {
        let (amount, lock) = if let Some(mut lock) = lock {
            let unlocked_amount = lock.unlock(current_timestamp) as u128;
            (
                amount + unlocked_amount * 10u128.pow(asset_id.decimals() as u32),
                Some(lock),
            )
        } else {
            (amount, lock)
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
        if self.amount >= amount {
            self.amount -= amount;
            amount
        } else {
            let amount_removed = amount - self.amount;
            self.amount = 0;
            amount_removed
        }
    }
    /// Unlock all possible tokens and returns new amount.
    pub fn unlock(&mut self, current_timestamp: TimestampSec) -> u128 {
        if let Some(lock) = &mut self.lock {
            self.amount += lock.unlock(current_timestamp) as u128
                * 10u128.pow(self.asset_id.decimals() as u32);
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

/// TODO: Refactor from tuples into structs.
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum Asset {
    Near,
    /// Account id and FT decimals.
    FT(AccountId, u8),
    /// TODO: Verify that only account_id is enough.
    NFT(AccountId, TokenId, ApprovalId),
}

impl Asset {
    pub fn new_near() -> Self {
        Self::Near
    }
    pub fn new_ft(account_id: AccountId, decimals: u8) -> Self {
        Self::FT(account_id, decimals)
    }
    pub fn new_nft(account_id: AccountId, token_id: String, approval_id: Option<u64>) -> Self {
        Self::NFT(account_id, token_id, approval_id)
    }
    pub fn decimals(&self) -> u8 {
        match &self {
            Self::Near => 24,
            Self::FT(_, decimals) => *decimals,
            _ => 0,
        }
    }
}

impl PartialEq for Asset {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::FT(l0, _), Self::FT(r0, _)) => *l0 == *r0,
            (Self::NFT(l0, l1, _), Self::NFT(r0, r1, _)) => *l0 == *r0 && *l1 == *r1,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl Contract {
    pub fn partition(&self, partition_id: u16) -> Option<TreasuryPartition> {
        self.treasury_partition.get(&partition_id)
    }
    pub fn add_partition(&mut self, partition: TreasuryPartition) -> u16 {
        self.partition_last_id += 1;
        self.treasury_partition
            .insert(&self.partition_last_id, &partition);
        self.partition_last_id
    }
    pub fn remove_partition(&mut self, partition_id: u16) -> Option<TreasuryPartition> {
        self.treasury_partition.remove(&partition_id)
    }
}
