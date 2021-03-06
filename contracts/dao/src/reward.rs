use library::{derive_from_versioned, derive_into_versioned};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::env::panic_str;
use near_sdk::serde::Serialize;
use near_sdk::AccountId;

use crate::internal::utils::current_timestamp_sec;
use crate::wallet::Wallet;
use crate::workflow::InternalDaoActionError;
use crate::{contract::*, AssetId, RewardId, RoleId};
use crate::{treasury::TreasuryPartition, TimestampSec};

derive_into_versioned!(Reward, VersionedReward, V1);
derive_from_versioned!(VersionedReward, Reward, V1);

#[derive(BorshDeserialize, BorshSerialize)]
pub enum VersionedReward {
    V1(Reward),
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Reward {
    pub name: String,
    pub group_id: u16,
    /// Role id in the group.
    pub role_id: u16,
    /// Partition from which the assets are taken.
    /// Partition must have all defined assets.
    pub partition_id: u16,
    /// Defines reward asset unit:
    /// - for `RewardType::Wage(seconds)` the unit is time.
    /// - for `RewardType::UserActivity(activity_ids)` the unit is activity done.
    /// Currently type: `RewardType::UserActivity(_)` is active for anyone regardless role and group.
    r#type: RewardType,
    /// Defines unique asset per unit.
    reward_amounts: Vec<(AssetId, u128)>,
    /// Timestamp reward is valid from.
    time_valid_from: u64,
    /// Timestamp reward is valid to.
    time_valid_to: u64,
}

impl Reward {
    pub fn new(
        name: String,
        group_id: u16,
        role_id: u16,
        partition_id: u16,
        r#type: RewardType,
        reward_amounts: Vec<(AssetId, u128)>,
        time_valid_from: TimestampSec,
        time_valid_to: TimestampSec,
    ) -> Self {
        Self {
            name,
            group_id,
            role_id,
            partition_id,
            r#type,
            reward_amounts,
            time_valid_from,
            time_valid_to,
        }
    }
    pub fn reward_amounts(&self) -> &[(AssetId, u128)] {
        self.reward_amounts.as_slice()
    }
    /// Return amount of total available wage asset reward at the current timestamp.
    /// Panics if reward type is not Wage.
    pub fn available_wage_amount(
        &self,
        asset_id: u8,
        timestamp_now: TimestampSec,
        timestamp_from: TimestampSec,
    ) -> u128 {
        let amount =
            if let Some((_, amount)) = self.reward_amounts.iter().find(|(r, _)| *r == asset_id) {
                amount
            } else {
                return 0;
            };
        match self.r#type {
            RewardType::Wage(ref wage) => {
                let seconds_passed = std::cmp::min(timestamp_now, self.time_valid_to)
                    - std::cmp::max(self.time_valid_from, timestamp_from);
                let units = seconds_passed / wage.unit_seconds as u64;
                amount.checked_mul(units as u128).unwrap_or(u128::MAX)
            }
            RewardType::UserActivity(_) => panic_str("fatal - invalid reward type"),
        }
    }
    pub fn is_valid(&self, current_timestamp: TimestampSec) -> bool {
        self.time_valid_from <= current_timestamp && current_timestamp <= self.time_valid_to
    }
    pub fn rewarded_activities(&self) -> Vec<u8> {
        match &self.r#type {
            RewardType::UserActivity(reward) => reward.activity_ids.clone(),
            _ => panic_str("fatal - invalid reward type"),
        }
    }
    pub fn get_reward_type(&self) -> RewardTypeIdent {
        self.r#type.get_ident()
    }
    /// Return amount of asset per executed activity.
    ///
    /// Panics if:
    /// - reward type is not `RewardType::UserActivity`
    /// - `asset` is not defined in reward
    pub fn reward_per_one_execution(&self, asset_id: u8) -> u128 {
        let amount = match self.r#type {
            RewardType::Wage(_) => panic_str("fatal - invalid reward type"),
            RewardType::UserActivity(_) => {
                let asset = self
                    .reward_amounts
                    .iter()
                    .find(|(r, _)| *r == asset_id)
                    .expect("fatal - asset not found");
                asset.1
            }
        };
        amount
    }
    pub fn set_time_valid_to(&mut self, time_valid_to: TimestampSec) {
        self.time_valid_to = time_valid_to;
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum RewardType {
    /// Unit is amount of provided seconds.
    Wage(RewardWage),
    /// TODO: Implementation.
    /// Activity id of done activities: Eg. voting, staking ...
    UserActivity(RewardUserActivity),
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct RewardWage {
    /// Amount of seconds define one unit.
    /// Must be > 0.
    pub unit_seconds: u16,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct RewardUserActivity {
    pub activity_ids: Vec<u8>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum RewardTypeIdent {
    Wage,
    UserActivity,
}

impl RewardType {
    pub fn new_wage(unit_seconds: u16) -> Self {
        RewardType::Wage(RewardWage { unit_seconds })
    }
    pub fn new_user_activity(activity_ids: Vec<u8>) -> Self {
        RewardType::UserActivity(RewardUserActivity { activity_ids })
    }
    pub fn get_ident(&self) -> RewardTypeIdent {
        match self {
            RewardType::Wage(_) => RewardTypeIdent::Wage,
            RewardType::UserActivity(_) => RewardTypeIdent::UserActivity,
        }
    }
}

pub enum RewardActivity {
    AcceptedProposal,
    Vote,
    Delegate,
    TransitiveDelegate,
    Activity,
}

impl TryFrom<u8> for RewardActivity {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::AcceptedProposal),
            1 => Ok(Self::Vote),
            2 => Ok(Self::Delegate),
            3 => Ok(Self::TransitiveDelegate),
            4 => Ok(Self::Activity),
            _ => Err("invalid reward activity id"),
        }
    }
}

impl From<RewardActivity> for u8 {
    fn from(r: RewardActivity) -> Self {
        match r {
            RewardActivity::AcceptedProposal => 0,
            RewardActivity::Vote => 1,
            RewardActivity::Delegate => 2,
            RewardActivity::TransitiveDelegate => 3,
            RewardActivity::Activity => 4,
        }
    }
}

impl PartialEq<u8> for RewardActivity {
    fn eq(&self, other: &u8) -> bool {
        match self {
            RewardActivity::AcceptedProposal => *other == 0,
            RewardActivity::Vote => *other == 1,
            RewardActivity::Delegate => *other == 2,
            RewardActivity::TransitiveDelegate => *other == 3,
            RewardActivity::Activity => *other == 4,
        }
    }
}

impl Contract {
    /// Add new reward entry into DAO rewards.
    /// Also add this reward in affected user wallets and related group.
    pub fn reward_add(&mut self, reward: Reward) -> Result<u16, InternalDaoActionError> {
        let partition = self
            .treasury_partition
            .get(&reward.partition_id)
            .ok_or_else(|| InternalDaoActionError("partition not found".into()))?
            .into();
        if !self.validate_reward_assets(&reward, &partition) {
            return Err(InternalDaoActionError(
                "partion does not have all required assets".into(),
            ));
        }
        if reward.time_valid_from >= reward.time_valid_to {
            return Err(InternalDaoActionError(
                "reward's time valid from must be smaller than time valid to".into(),
            ));
        }
        self.reward_last_id += 1;

        // In case of group_id == 0 all wallet reward entries are created lazily.
        if reward.group_id > 0 {
            let mut group = self
                .groups
                .get(&reward.group_id)
                .ok_or_else(|| InternalDaoActionError("group not found".into()))?;
            let rewarded_users = if reward.role_id == 0 {
                group.get_members_accounts()
            } else {
                self.get_group_members_with_role(reward.group_id, &group, reward.role_id)
            };
            let mut reward_assets: Vec<AssetId> = reward
                .reward_amounts()
                .iter()
                .map(|(a, _)| a.to_owned())
                .collect();
            reward_assets.sort_unstable();
            let len_before = reward_assets.len();
            reward_assets.dedup();
            if len_before != reward_assets.len() {
                return Err(InternalDaoActionError("duplicate assets".into()));
            }
            let current_timestamp = current_timestamp_sec();
            for user in rewarded_users {
                self.add_wallet_reward(
                    self.reward_last_id,
                    reward.get_reward_type(),
                    &user,
                    reward_assets.clone(),
                    current_timestamp,
                );
            }
            group.add_new_reward(self.reward_last_id, reward.role_id);
            self.groups.insert(&reward.group_id, &group);
        } else if reward.get_reward_type() != RewardTypeIdent::UserActivity {
            return Err(InternalDaoActionError(
                "only activity rewards can be defined for anyone".into(),
            ));
        }
        if reward.get_reward_type() == RewardTypeIdent::UserActivity {
            for activity_id in reward.rewarded_activities() {
                if let Some(mut rewards) = self.cache_reward_activity.get(&activity_id) {
                    rewards.push(self.reward_last_id);
                    self.cache_reward_activity.insert(&activity_id, &rewards);
                }
            }
        }
        self.rewards.insert(&self.reward_last_id, &reward.into());
        Ok(self.reward_last_id)
    }

    /// Update Reward.
    /// Currently only updates valid to time.
    pub fn reward_update(
        &mut self,
        id: u16,
        time_valid_to: u64,
    ) -> Result<(), InternalDaoActionError> {
        if let Some(reward) = self.rewards.get(&id) {
            let mut reward: Reward = reward.into();
            if reward.time_valid_from >= time_valid_to {
                return Err(InternalDaoActionError(
                    "reward's time valid from must be smaller than time valid to".into(),
                ));
            }
            reward.set_time_valid_to(time_valid_to);
            self.rewards.insert(&id, &reward.into());
        }
        Ok(())
    }

    /// Validate that defined assets in rewards are defined in treasury partition.
    pub fn validate_reward_assets(&self, reward: &Reward, partition: &TreasuryPartition) -> bool {
        let partition_assets = partition.assets();
        for (rew_asset, _) in reward.reward_amounts() {
            if partition_assets
                .iter()
                .any(|el| el.asset_id() == *rew_asset)
            {
                continue;
            } else {
                return false;
            }
        }
        true
    }

    /// Add Reward data to the `account_id`'s wallet.
    pub fn add_wallet_reward(
        &mut self,
        reward_id: u16,
        reward_type: RewardTypeIdent,
        account_id: &AccountId,
        assets: Vec<AssetId>,
        current_timestamp: TimestampSec,
    ) {
        let mut wallet = self.get_wallet(account_id);
        wallet.add_reward(reward_id, reward_type, current_timestamp, assets);
        self.wallets.insert(account_id, &wallet.into());
    }
    /// Add multiple Rewards to the `account_is` wallet.
    pub fn add_wallet_rewards(
        &mut self,
        account_id: &AccountId,
        rewards: Vec<(RewardId, Reward)>,
        current_timestamp: TimestampSec,
    ) {
        let mut wallet = self.get_wallet(account_id);
        for (reward_id, reward) in rewards {
            let assets: Vec<AssetId> = reward
                .reward_amounts()
                .iter()
                .map(|(a, _)| a.to_owned())
                .collect();
            wallet.add_reward(
                reward_id,
                reward.get_reward_type(),
                current_timestamp,
                assets,
            );
        }
        self.wallets.insert(account_id, &wallet.into());
    }
    /// Remove `rewards` for `account_id`'s wallet.
    /// Internally it updates wallet stats so the rewards up to the `current_timestamp` are kept.
    pub fn remove_wallet_reward(
        &mut self,
        account_id: &AccountId,
        rewards: &[(RewardId, RoleId)],
        current_timestamp: TimestampSec,
    ) {
        if let Some(versioned_wallet) = self.wallets.get(account_id) {
            let mut wallet: Wallet = versioned_wallet.into();
            for (reward_id, _) in rewards {
                wallet.set_reward_timestamp_removed(*reward_id, current_timestamp);
            }
            self.wallets.insert(account_id, &wallet.into());
        }
    }

    /// Register executed activity to `account_id`'s Wallet for each reward.
    /// Also register activity rewards for anyone in the `account_id` wallet.
    pub fn register_executed_activity(&mut self, account_id: &AccountId, activity_id: u8) {
        let mut wallet: Wallet = self.get_wallet(account_id);
        let valid_rewards = self.valid_reward_list_for_activity(activity_id);
        let current_timestamp = current_timestamp_sec();
        for (id, reward) in valid_rewards {
            if reward.group_id == 0 && wallet.wallet_reward(id).is_none() {
                wallet.add_reward(
                    id,
                    reward.get_reward_type(),
                    current_timestamp,
                    reward.reward_amounts().iter().map(|(a, _)| *a).collect(),
                );
            }
            wallet.add_executed_activity(id);
        }
        if !wallet.rewards().is_empty() {
            self.wallets.insert(account_id, &wallet.into());
        }
    }

    /// Return list of valid reward ids that reward `activity_id`.
    /// Also update cache - remove expired rewards.
    pub fn valid_reward_list_for_activity(&mut self, activity_id: u8) -> Vec<(u16, Reward)> {
        let mut reward_list = vec![];
        let mut expired_reward_ids = vec![];
        let mut rewards = self
            .cache_reward_activity
            .get(&activity_id)
            .expect("fatal - activity not defined");
        let current_timestamp = current_timestamp_sec();
        for id in rewards.iter() {
            let reward: Reward = self
                .rewards
                .get(id)
                .expect("fatal - reward not defined")
                .into();
            if reward.is_valid(current_timestamp) {
                reward_list.push((*id, reward));
            } else {
                expired_reward_ids.push(*id);
            }
        }
        if !expired_reward_ids.is_empty() {
            for id in expired_reward_ids {
                let pos = rewards.iter().position(|e| *e == id).unwrap();
                rewards.swap_remove(pos);
            }
            self.cache_reward_activity.insert(&activity_id, &rewards);
        }
        reward_list
    }
}
