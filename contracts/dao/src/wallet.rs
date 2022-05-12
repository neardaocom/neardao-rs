use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env, ext_contract,
    json_types::U128,
    near_bindgen,
    serde::Serialize,
    AccountId, Promise, PromiseResult,
};

use crate::{
    constants::TGAS, core::*, derive_from_versioned, derive_into_versioned,
    error::ERR_PROMISE_INVALID_RESULTS_COUNT, internal::utils::current_timestamp_sec,
    reward::Reward, treasury::Asset, TimestampSec,
};

#[ext_contract(ext_self)]
trait ExtWallet {
    /// Rollback if promise failed.
    fn withdraw_rollback(account_id: AccountId, asset: Asset, amount: u128);
}

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

impl Wallet {
    pub fn new() -> Self {
        Self {
            rewards: Vec::default(),
            failed_withdraws: Vec::default(),
        }
    }
    /// Add reward only if reward_id is unique.
    /// Return true if sucessful.
    /// Caller is responsible to:
    /// - provide unique assets.
    /// - provide correct `assets` values for `reward_id`, that means referenced reward has all provided assets.
    /// These invariants are not checked.
    pub fn add_reward(
        &mut self,
        reward_id: u16,
        current_timestamp: TimestampSec,
        assets: Vec<Asset>,
    ) -> bool {
        if self.find_reward_pos(reward_id).is_some() {
            return false;
        }
        let reward = WalletReward::new(reward_id, current_timestamp, assets);
        self.rewards.push(reward);
        true
    }
    /// Return total sum of withdrawn `asset` from `reward_id` reward.
    /// Panic if `reward_id` is not found.
    pub fn amount_reward_withdrawn(&self, reward_id: u16, asset: &Asset) -> u128 {
        let pos = self.find_reward_pos(reward_id).expect("reward not found");
        let reward = self.rewards.get(pos).unwrap();
        let asset = reward.withdraw_stat(asset);
        asset.amount
    }
    /// Remove previous amount that failed to be withdrawn and return it.
    /// This amount is already subtracted from the partition.
    pub fn take_failed_withdraw_amount(&mut self, asset: &Asset) -> u128 {
        if let Some(pos) = self.find_failed_withdraw_pos(asset) {
            self.failed_withdraws.swap_remove(pos).1
        } else {
            0
        }
    }
    pub fn remove_reward(&mut self, reward_id: u16) {
        let pos = self.find_reward_pos(reward_id).expect("reward not found");
        self.rewards.swap_remove(pos);
    }
    /// Update withdraw stat for `reward_id`'s `asset` with `amount` withdrawn.
    /// Panic if `reward_id` is not found.
    pub fn withdraw_reward(
        &mut self,
        reward_id: u16,
        asset: &Asset,
        amount: u128,
        current_timestamp: TimestampSec,
    ) {
        let pos = self.find_reward_pos(reward_id).expect("reward not found");
        let reward = self.rewards.get_mut(pos).unwrap();
        reward.add_withdrawn_amount(asset, amount, current_timestamp);
    }
    /// Adds rewards which failed to be withdraw.
    pub fn withdraw_reward_failed(&mut self, asset: Asset, amount: u128) {
        if let Some(pos) = self.find_failed_withdraw_pos(&asset) {
            self.failed_withdraws.get_mut(pos).unwrap().1 += amount;
        } else {
            self.failed_withdraws.push((asset, amount));
        }
    }

    #[inline]
    fn find_reward_pos(&self, reward_id: u16) -> Option<usize> {
        self.rewards.iter().position(|r| r.reward_id == reward_id)
    }
    #[inline]
    fn find_failed_withdraw_pos(&self, asset: &Asset) -> Option<usize> {
        self.failed_withdraws.iter().position(|(a, _)| a == asset)
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct WalletReward {
    reward_id: u16,
    time_added: TimestampSec,
    /// Sums of all withdrawn assets from reward.
    /// All assets MUST be defined in the reward.
    withdraw_stats: Vec<WithdrawStats>,
}

impl WalletReward {
    pub fn new(reward_id: u16, time_added: TimestampSec, assets: Vec<Asset>) -> Self {
        let withdraw_stats = assets.into_iter().map(|a| WithdrawStats::new(a)).collect();
        Self {
            reward_id,
            time_added,
            withdraw_stats,
        }
    }
    pub fn withdraw_stats(&self) -> &[WithdrawStats] {
        self.withdraw_stats.as_slice()
    }
    fn find_asset_pos(&self, asset: &Asset) -> Option<usize> {
        self.withdraw_stats
            .iter()
            .position(|s| s.asset_id == *asset)
    }
    pub fn withdraw_stat(&self, asset: &Asset) -> &WithdrawStats {
        let pos = self.find_asset_pos(asset).expect("asset stat not found");
        self.withdraw_stats.get(pos).unwrap()
    }
    pub fn add_withdrawn_amount(&mut self, asset: &Asset, amount: u128, current_timestamp: u64) {
        let pos = self.find_asset_pos(asset).expect("asset stat not found");
        let stats = self.withdraw_stats.get_mut(pos).unwrap();
        stats.amount += amount;
        stats.timestamp_last_withdraw += current_timestamp;
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct WithdrawStats {
    pub asset_id: Asset,
    pub amount: u128,
    pub timestamp_last_withdraw: TimestampSec,
}

impl WithdrawStats {
    pub fn new(asset_id: Asset) -> Self {
        Self {
            asset_id,
            amount: 0,
            timestamp_last_withdraw: 0,
        }
    }
}

#[near_bindgen]
impl Contract {
    // TODO: Allow to define max withdraw amount per reward?
    /// Withdraw all `asset` rewards defined by reward_ids from caller's wallet.
    /// Panics if any provided reward_id is invalid.
    /// Return actually withdrawn amount.
    pub fn withdraw_rewards(&mut self, reward_ids: Vec<u16>, asset: Asset) -> U128 {
        self.debug_log.push(format!(
            "withdraw - current timestamp: {}; ",
            current_timestamp_sec(),
        ));
        let mut total_withdrawn = 0;
        let caller = env::predecessor_account_id();
        let current_timestamp = current_timestamp_sec();
        let mut wallet = self.get_wallet(&caller);
        for reward_id in reward_ids {
            let reward: Reward = self
                .rewards
                .get(&reward_id)
                .expect("reward not found in the dao")
                .into();
            // Reward available in rewards.
            let amount_available_reward = reward.available_asset_amount(&asset, current_timestamp);
            if amount_available_reward > 0 {
                // Check for already withdrawn by user in his wallet.
                let amount_already_claimed = wallet.amount_reward_withdrawn(reward_id, &asset);
                let claimable_reward = amount_available_reward - amount_already_claimed;

                // Try to withdraw possible amount from the partition.
                let mut partition = self.treasury_partition.get(&reward.partition_id).unwrap();
                let available_partition_amount = partition.remove_amount(&asset, claimable_reward);
                self.treasury_partition
                    .insert(&reward.partition_id, &partition);
                wallet.withdraw_reward(
                    reward_id,
                    &asset,
                    available_partition_amount,
                    current_timestamp,
                );
                total_withdrawn += amount_available_reward;
            }
        }
        total_withdrawn += wallet.take_failed_withdraw_amount(&asset);
        self.wallets.insert(&caller, &wallet.into());
        // Send reward to the caller.
        if total_withdrawn > 0 {
            self.send_reward(caller, asset, total_withdrawn);
        }
        total_withdrawn.into()
    }
    /// Calculate available rewards for `account_id`.
    pub fn available_rewards(&self, account_id: AccountId) {
        todo!();
    }

    #[private]
    pub fn withdraw_rollback(&mut self, account_id: AccountId, asset: Asset, amount: u128) {
        assert_eq!(
            env::promise_results_count(),
            1,
            "{}",
            ERR_PROMISE_INVALID_RESULTS_COUNT
        );
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                self.debug_log.push(format!(
                    "{} withdraw {} amount success; ",
                    account_id.as_str(),
                    amount
                ));
            }
            PromiseResult::Failed => {
                self.debug_log.push(format!(
                    "{} withdraw {} amount failed; ",
                    account_id.as_str(),
                    amount
                ));
                let mut wallet = self.get_wallet(&account_id);
                wallet.withdraw_reward_failed(asset, amount);
                self.wallets.insert(&account_id, &wallet.into());
            }
        }
    }
}

impl Contract {
    pub fn get_wallet(&mut self, account_id: &AccountId) -> Wallet {
        self.wallets
            .get(account_id)
            .unwrap_or_else(|| VersionedWallet::Current(Wallet::new()))
            .into()
    }
    pub fn send_reward(&mut self, account_id: AccountId, asset: Asset, amount: u128) {
        match asset {
            Asset::Near => {
                Promise::new(account_id).transfer(amount);
            }
            Asset::FT(ft) => {
                let args = format!(
                    "{{\"receiver_id\":\"{}\",\"amount\":\"{}\",\"memo\":null}}",
                    account_id, amount
                );
                Promise::new(ft.account_id.clone())
                    .function_call("ft_transfer".into(), args.into_bytes(), 1, TGAS * 10)
                    .then(ext_self::withdraw_rollback(
                        account_id,
                        Asset::new_ft(ft.account_id, ft.decimals),
                        amount,
                        env::current_account_id(),
                        0,
                        TGAS * 10,
                    ));
            }
            Asset::NFT(nft) => {
                let approval_id_string = if let Some(approval_id) = nft.approval_id.clone() {
                    approval_id.to_string()
                } else {
                    "null".to_string()
                };
                let args = format!(
                    "{{\"receiver_id\":\"{}\",\"token_id\":\"{}\",\"approval_id\":{},\"memo\":null}}",
                    account_id, &nft.token_id, approval_id_string
                );
                Promise::new(nft.account_id.clone())
                    .function_call("nft_transfer".into(), args.into_bytes(), 1, TGAS * 10)
                    .then(ext_self::withdraw_rollback(
                        account_id,
                        Asset::new_nft(nft.account_id, nft.token_id, nft.approval_id),
                        amount,
                        env::current_account_id(),
                        0,
                        TGAS * 10,
                    ));
            }
        };
    }
}
