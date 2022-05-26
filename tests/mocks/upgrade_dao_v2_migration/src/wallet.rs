use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    json_types::U128,
    serde::Serialize,
};

use crate::{derive_from_versioned, derive_into_versioned, treasury::Asset, TimestampSec};

derive_into_versioned!(Wallet, VersionedWallet);
derive_from_versioned!(VersionedWallet, Wallet);

#[derive(BorshDeserialize, BorshSerialize)]
pub enum VersionedWallet {
    Current(Wallet),
}

/// Wallet keep info about owner's rewards.
/// Rewards which are active or owner has not claimed all rewards yet
/// are being kept in the wallet.
#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Wallet {
    /// Currently provided unique rewards.
    rewards: Vec<WalletReward>,
    /// Rewards that failed to be withdrawn. These are immediately available to be withdrawn again.
    failed_withdraws: Vec<(Asset, u128)>,
}

/// Reference to a Reward defined in contract.
/// Store data about when was added/removed and withdraw stats.
#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct WalletReward {
    reward_id: u16,
    time_added: TimestampSec,
    time_removed: Option<TimestampSec>,
    /// Collection of all withdrawn stats per asset from reward.
    /// All assets MUST be defined in the reward.
    /// In case of UserActivityReward its counter of executed activities.
    /// Withdraw asset resets it.
    withdraw_stats: Vec<WithdrawStats>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum WithdrawStats {
    Wage(WageStats),
    Activity(ActivityStats),
}

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct WageStats {
    pub asset_id: Asset,
    pub amount: u128,
    pub timestamp_last_withdraw: TimestampSec,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ActivityStats {
    pub asset_id: Asset,
    /// Count of all executions that have not been withdrawn yet.
    pub executed_count: u16,
    /// Total count of withdrawn executions.
    pub total_withdrawn_count: u16,
    pub timestamp_last_withdraw: TimestampSec,
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ClaimableReward {
    pub asset: Asset,
    pub reward_id: u16,
    pub amount: U128,
    pub partition_id: u16,
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ClaimableRewards {
    pub claimable_rewards: Vec<ClaimableReward>,
    pub failed_withdraws: Vec<(Asset, U128)>,
}
