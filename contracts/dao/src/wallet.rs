use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env, ext_contract,
    json_types::U128,
    near_bindgen, require,
    serde::Serialize,
    AccountId, Promise, PromiseResult,
};

use crate::{
    constants::TGAS,
    core::*,
    derive_from_versioned, derive_into_versioned,
    error::ERR_PROMISE_INVALID_RESULTS_COUNT,
    internal::utils::current_timestamp_sec,
    reward::{Reward, RewardTypeIdent},
    treasury::Asset,
    TimestampSec,
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
        reward_type: RewardTypeIdent,
        current_timestamp: TimestampSec,
        assets: Vec<Asset>,
    ) -> bool {
        if self.find_reward_pos(reward_id).is_some() {
            return false;
        }
        let reward = WalletReward::new(reward_id, reward_type, current_timestamp, assets);
        self.rewards.push(reward);
        true
    }
    /// Return total sum of withdrawn `asset` from Wage `reward_id` reward.
    /// Panic if `reward_id` is not found or is invalid type of reward.
    /// Its up to the caller to verify what the reward type.
    pub fn amount_wage_withdrawn(&self, reward_id: u16, asset: &Asset) -> u128 {
        let pos = self.find_reward_pos(reward_id).expect("reward not found");
        let reward = &self.rewards[pos];
        let stat = reward.withdraw_stat(asset);
        stat.wage_as_ref()
            .expect("fatal - invalid reward type")
            .amount
    }
    /// Return amount of executed activity for `reward_id` reward.
    ///
    /// Panics if:
    /// - reward is not found
    /// - reward type is not `RewardTypeIdent::UserActivity`
    pub fn user_activity_executed_count(&self, reward_id: u16, asset: &Asset) -> u16 {
        let pos = self.find_reward_pos(reward_id).expect("reward not found");
        let reward = &self.rewards[pos];
        let stat = reward.withdraw_stat(asset);
        stat.activity_as_ref()
            .expect("fatal - invalid reward type")
            .executed_count
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
    /// Update withdraw stats for `reward_id`'s `asset` with `amount` withdrawn.
    /// Panic if:
    /// - `reward_id` is not found
    /// - reward type is not wage
    /// - `asset` is not found
    pub fn withdraw_wage(
        &mut self,
        reward_id: u16,
        asset: &Asset,
        amount: u128,
        current_timestamp: TimestampSec,
    ) {
        let pos = self.find_reward_pos(reward_id).expect("reward not found");
        let reward = &mut self.rewards[pos];
        reward.wage_withdraw_amount(asset, amount, current_timestamp);
    }
    /// Update withdraw stats for `reward_id`'s `asset` with `amount` withdrawn.
    /// Panic if:
    /// - `reward_id` is not found
    /// - reward type is not activity
    /// - `asset` is not found
    pub fn withdraw_activity(
        &mut self,
        reward_id: u16,
        asset: &Asset,
        amount: u16,
        current_timestamp: TimestampSec,
    ) {
        let pos = self.find_reward_pos(reward_id).expect("reward not found");
        let reward = &mut self.rewards[pos];
        reward.activity_withdrawn(asset, amount, current_timestamp);
    }
    /// Add rewards which failed to be withdraw.
    pub fn withdraw_reward_failed(&mut self, asset: Asset, amount: u128) {
        if let Some(pos) = self.find_failed_withdraw_pos(&asset) {
            self.failed_withdraws[pos].1 += amount;
        } else {
            self.failed_withdraws.push((asset, amount));
        }
    }
    /// Add one to execution counter for `reward_id`.
    pub fn add_executed_activity(&mut self, reward_id: u16) {
        if let Some(pos) = self.find_reward_pos(reward_id) {
            self.rewards[pos].activity_executed();
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
    /// Vec of all withdrawn stats per asset from reward.
    /// All assets MUST be defined in the reward.
    /// In case of UserActivityReward its counter of executed activities.
    /// Withdraw asset resets it.
    withdraw_stats: Vec<WithdrawStats>,
}

impl WalletReward {
    pub fn new(
        reward_id: u16,
        reward_type: RewardTypeIdent,
        time_added: TimestampSec,
        assets: Vec<Asset>,
    ) -> Self {
        require!(assets.len() > 0, "empty assets as rewards");
        let withdraw_stats = assets
            .into_iter()
            .map(|a| WithdrawStats::new(a, reward_type))
            .collect();
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
        self.withdraw_stats.iter().position(|s| *s == *asset)
    }
    pub fn withdraw_stat(&self, asset: &Asset) -> &WithdrawStats {
        let pos = self.find_asset_pos(asset).expect("asset stat not found");
        &self.withdraw_stats[pos]
    }
    pub fn wage_withdraw_amount(&mut self, asset: &Asset, amount: u128, current_timestamp: u64) {
        let pos = self.find_asset_pos(asset).expect("asset stat not found");
        let stats = self.withdraw_stats[pos]
            .wage_as_mut()
            .expect("fatal - valid reward type");
        stats.amount += amount;
        stats.timestamp_last_withdraw += current_timestamp;
    }
    /// Update counters for activity reward type.
    /// Panics if:
    /// - `asset` is not found
    /// - reward is not type of activity
    /// - `amount` is greater than activity execution count
    pub fn activity_withdrawn(&mut self, asset: &Asset, amount: u16, current_timestamp: u64) {
        let pos = self.find_asset_pos(asset).expect("asset stat not found");
        let stats = self.withdraw_stats[pos]
            .activity_as_mut()
            .expect("fatal - valid reward type");
        stats.executed_count = stats
            .executed_count
            .checked_sub(amount)
            .expect("fatal - activity withdraw");
        stats.total_withdrawn_count += amount;
        stats.timestamp_last_withdraw += current_timestamp;
    }
    pub fn activity_executed(&mut self) {
        for stat in self.withdraw_stats.iter_mut() {
            stat.activity_as_mut()
                .expect("fatal - invalid reward type")
                .executed_count += 1;
        }
    }
    pub fn reward_type(&self) -> RewardTypeIdent {
        let stat = &self.withdraw_stats[0];
        stat.get_reward_type()
    }
}
#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum WithdrawStats {
    Wage(WageStats),
    Activity(ActivityStats),
}

impl WithdrawStats {
    pub fn new(asset_id: Asset, reward_type: RewardTypeIdent) -> Self {
        match reward_type {
            RewardTypeIdent::Wage => WithdrawStats::Wage(WageStats {
                asset_id,
                amount: 0,
                timestamp_last_withdraw: 0,
            }),
            RewardTypeIdent::UserActivity => WithdrawStats::Activity(ActivityStats {
                asset_id,
                executed_count: 0,
                total_withdrawn_count: 0,
                timestamp_last_withdraw: 0,
            }),
        }
    }
    pub fn get_reward_type(&self) -> RewardTypeIdent {
        match self {
            WithdrawStats::Wage(_) => RewardTypeIdent::Wage,
            WithdrawStats::Activity(_) => RewardTypeIdent::UserActivity,
        }
    }
    pub fn wage_as_ref(&self) -> Option<&WageStats> {
        match self {
            WithdrawStats::Wage(s) => Some(s),
            WithdrawStats::Activity(_) => None,
        }
    }
    pub fn wage_as_mut(&mut self) -> Option<&mut WageStats> {
        match self {
            WithdrawStats::Wage(s) => Some(s),
            WithdrawStats::Activity(_) => None,
        }
    }
    pub fn activity_as_ref(&self) -> Option<&ActivityStats> {
        match self {
            WithdrawStats::Wage(_) => None,
            WithdrawStats::Activity(a) => Some(a),
        }
    }
    pub fn activity_as_mut(&mut self) -> Option<&mut ActivityStats> {
        match self {
            WithdrawStats::Wage(_) => None,
            WithdrawStats::Activity(a) => Some(a),
        }
    }
    pub fn last_time_withdrawn(&self) -> TimestampSec {
        match self {
            WithdrawStats::Wage(s) => s.timestamp_last_withdraw,
            WithdrawStats::Activity(a) => a.timestamp_last_withdraw,
        }
    }
}

impl PartialEq<Asset> for WithdrawStats {
    fn eq(&self, asset: &Asset) -> bool {
        match self {
            Self::Wage(l0) => l0.asset_id == *asset,
            Self::Activity(l0) => l0.asset_id == *asset,
        }
    }
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

#[near_bindgen]
impl Contract {
    // TODO: Allow to define max withdraw amount per reward?
    /// Withdraw all `asset` rewards defined by reward_ids from caller's wallet.
    /// Panics if any provided reward_id is invalid.
    /// Return actually withdrawn amount.
    pub fn withdraw_rewards(&mut self, reward_ids: Vec<u16>, asset: Asset) -> U128 {
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
            // Find out max amount user is allowed to claim.
            let is_wage = matches!(reward.get_reward_type(), RewardTypeIdent::Wage);
            let (claimable_reward, amount_per_activity) = if is_wage {
                let amount_available_reward =
                    reward.available_wage_amount(&asset, current_timestamp);
                let amount_already_claimed = wallet.amount_wage_withdrawn(reward_id, &asset);
                (amount_available_reward - amount_already_claimed, 0)
            } else {
                let generated_amount =
                    wallet.user_activity_executed_count(reward_id, &asset) as u128;
                let amount_per_activity = reward.reward_per_one_execution(&asset);
                (generated_amount * amount_per_activity, amount_per_activity)
            };
            // Nothing to claim - check next reward.
            if claimable_reward == 0 {
                continue;
            }
            // Get maximal claimable amount from treasury.
            let mut partition = self.treasury_partition.get(&reward.partition_id).unwrap();
            let currently_available_amount =
                partition.remove_amount(&asset, amount_per_activity, claimable_reward);
            self.treasury_partition
                .insert(&reward.partition_id, &partition);
            // Update caller's wallet with actually withdrawn amounts.
            if is_wage {
                wallet.withdraw_wage(
                    reward_id,
                    &asset,
                    currently_available_amount,
                    current_timestamp,
                );
            } else {
                wallet.withdraw_activity(
                    reward_id,
                    &asset,
                    (currently_available_amount / amount_per_activity) as u16,
                    current_timestamp,
                );
            }
            total_withdrawn += currently_available_amount;
        }
        total_withdrawn += wallet.take_failed_withdraw_amount(&asset);
        self.wallets.insert(&caller, &wallet.into());
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
