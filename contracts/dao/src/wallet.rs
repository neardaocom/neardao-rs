use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    json_types::U128,
    near_bindgen, AccountId,
};

use crate::{core::*, treasury::Asset, TimestampSec};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Wallet {
    /// Currently provided rewards.
    rewards: Vec<WalletReward>,
}

impl Wallet {
    pub fn new() -> Self {
        todo!()
    }
    pub fn add_reward(&mut self) {}
    pub fn remove_reward(&mut self) {}
    pub fn withdraw_reward(&mut self, reward_id: u16) {}
    pub fn find_reward_pos(&self, reward_id: u16) -> Option<usize> {
        self.rewards.iter().position(|r| r.reward_id == reward_id)
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct WalletReward {
    reward_id: u16,
    time_added_from: TimestampSec,
    /// Sums of all withdrawn assets from reward.
    withdraw_stats: Vec<WithdrawStats>,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct WithdrawStats {
    asset_id: Asset,
    amount: u16,
}

#[near_bindgen]
impl Contract {
    /// Withdraw `asset` rewards from provided reward_ids from the wallet.
    /// Return actually withdrawn amount.
    pub fn withdraw(&mut self, reward_ids: Vec<u16>, asset: Asset) -> U128 {
        todo!()
    }
    /// Calculate available rewards for `account_id`.
    pub fn available_rewards(&self, account_id: AccountId) {}
}

impl Contract {
    pub fn add_wallet(&mut self) {}
    pub fn remove_wallet(&mut self) {}
    pub fn internal_withdraw(&mut self) {}
}
