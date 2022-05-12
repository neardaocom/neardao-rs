use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::env::panic_str;
use near_sdk::serde::Serialize;
use near_sdk::{require, AccountId};

use crate::internal::utils::current_timestamp_sec;
use crate::wallet::Wallet;
use crate::{core::*, derive_from_versioned, derive_into_versioned};
use crate::{
    treasury::{Asset, TreasuryPartition},
    TimestampSec,
};

derive_into_versioned!(Reward, VersionedReward);
derive_from_versioned!(VersionedReward, Reward);

#[derive(BorshDeserialize, BorshSerialize)]
pub enum VersionedReward {
    Current(Reward),
}

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Reward {
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
    reward_amounts: Vec<(Asset, u128)>,
    /// TODO: Unimplemented.
    time_valid_from: u64,
    /// TODO: Unimplemented.
    time_valid_to: u64,
}

impl Reward {
    /// TODO: Validations
    pub fn new(
        group_id: u16,
        role_id: u16,
        partition_id: u16,
        r#type: RewardType,
        reward_amounts: Vec<(Asset, u128)>,
        time_valid_from: TimestampSec,
        time_valid_to: TimestampSec,
    ) -> Self {
        Self {
            group_id,
            role_id,
            partition_id,
            r#type,
            reward_amounts,
            time_valid_from,
            time_valid_to,
        }
    }
    pub fn reward_amounts(&self) -> &[(Asset, u128)] {
        self.reward_amounts.as_slice()
    }
    /// TODO: Check edge cases in wasm to know max/min amounts.
    /// Return amount of total available wage asset reward at the current timestamp.
    /// Panics if reward type is not Wage.
    pub fn available_wage_amount(&self, asset: &Asset, current_timestamp: TimestampSec) -> u128 {
        let amount = if let Some((_, amount)) = self.reward_amounts.iter().find(|(r, _)| r == asset)
        {
            amount
        } else {
            return 0;
        };
        let amount = match self.r#type {
            RewardType::Wage(ref wage) => {
                let seconds_passed = current_timestamp - self.time_valid_from;
                let units = seconds_passed / wage.unit_seconds as u64;
                amount * units as u128
            }
            RewardType::UserActivity(_) => panic_str("fatal - invalid reward type"),
        };
        amount.into()
    }
    pub fn is_valid(&self, current_timestamp: TimestampSec) -> bool {
        self.time_valid_from <= current_timestamp && current_timestamp <= self.time_valid_to
    }
    pub fn is_defined_for_activity(&self, activity_id: u8) -> bool {
        match &self.r#type {
            RewardType::UserActivity(reward) => reward.activity_ids.contains(&activity_id),
            _ => false,
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
    pub fn reward_per_one_execution(&self, asset: &Asset) -> u128 {
        let amount = match self.r#type {
            RewardType::Wage(_) => panic_str("fatal - invalid reward type"),
            RewardType::UserActivity(_) => {
                let asset = self
                    .reward_amounts
                    .iter()
                    .find(|(r, _)| r == asset)
                    .expect("fatal - asset not found");
                asset.1
            }
        };
        amount
    }
}

/// TODO: Refactor from tuples into structs.
#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum RewardType {
    /// Unit is amount of provided seconds.
    Wage(RewardWage),
    /// TODO: Implementation.
    /// Activity id of done activities: Eg. voting, staking ...
    UserActivity(RewardUserActivity),
}

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct RewardWage {
    /// Amount of seconds define one unit.
    pub unit_seconds: u16,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct RewardUserActivity {
    pub activity_ids: Vec<u8>,
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

#[derive(Clone, Copy, PartialEq)]
pub enum RewardTypeIdent {
    Wage,
    UserActivity,
}

impl Contract {
    /// TODO: Error handling and validations.
    /// Add new reward entry into DAO rewards.
    /// Also add this reward in affected user wallets.
    pub fn add_reward(&mut self, reward: Reward) -> u16 {
        let partition = self
            .treasury_partition
            .get(&reward.partition_id)
            .expect("partition not found");

        assert!(
            self.validate_reward_assets(&reward, &partition),
            "partion does not have all required assets"
        );

        let group = self.groups.get(&reward.group_id).expect("group not found");
        let rewarded_users =
            self.get_group_members_with_role(reward.group_id, &group, reward.role_id);

        self.reward_last_id += 1;

        // Add reward to role members wallets.
        let mut reward_assets: Vec<Asset> = reward
            .reward_amounts()
            .into_iter()
            .map(|(a, _)| a.to_owned())
            .collect();

        // Check for duplicates.
        reward_assets.sort();
        let len_before = reward_assets.len();
        reward_assets.dedup();
        assert!(len_before == reward_assets.len(), "duplicate assets");
        let current_timestamp = current_timestamp_sec();
        for user in rewarded_users {
            assert!(
                self.add_reward_to_user_wallet(
                    self.reward_last_id,
                    reward.get_reward_type(),
                    &user,
                    reward_assets.clone(),
                    current_timestamp
                ),
                "failed to added reward to user wallet"
            );
        }
        self.rewards.insert(&self.reward_last_id, &reward.into());
        self.reward_last_id
    }
    /// TODO: Check it can be safely removed.
    pub fn remove_reward(&mut self, reward_id: u16) -> Option<Reward> {
        self.rewards.remove(&reward_id).map(|reward| reward.into())
    }

    /// Validate that defined assets in rewards are defined in treasury partition.
    pub fn validate_reward_assets(&self, reward: &Reward, partition: &TreasuryPartition) -> bool {
        let partition_assets = partition.assets();
        for (rew, _) in reward.reward_amounts() {
            if partition_assets.iter().any(|el| el.asset_id() == rew) {
                break;
            } else {
                return false;
            }
        }
        true
    }

    /// Add reward to the `account_id`.
    /// Return true if added.
    pub fn add_reward_to_user_wallet(
        &mut self,
        reward_id: u16,
        reward_type: RewardTypeIdent,
        account_id: &AccountId,
        assets: Vec<Asset>,
        current_timestamp: TimestampSec,
    ) -> bool {
        let mut wallet = self.get_wallet(account_id);
        let added = wallet.add_reward(reward_id, reward_type, current_timestamp, assets);
        self.wallets.insert(account_id, &wallet.into());
        added
    }

    pub fn register_executed_activity(&mut self, account_id: &AccountId, activity_id: u8) {
        if let Some(wallet) = self.wallets.get(account_id) {
            let mut wallet: Wallet = wallet.into();
            let valid_rewards = self.valid_reward_list_for_activity(activity_id);
            for reward_id in valid_rewards {
                wallet.add_executed_activity(reward_id);
            }
            self.wallets.insert(account_id, &wallet.into());
        }
    }

    /// Return list of valid reward ids that reward `activity_id`.
    pub fn valid_reward_list_for_activity(&self, activity_id: u8) -> Vec<u16> {
        let mut rewards = Vec::with_capacity(self.reward_last_id as usize / 2);
        let current_timestamp = current_timestamp_sec();
        for i in 1..=self.reward_last_id {
            if let Some(reward) = self.rewards.get(&i) {
                let reward: Reward = reward.into();
                if reward.is_valid(current_timestamp) && reward.is_defined_for_activity(activity_id)
                {
                    rewards.push(i);
                }
            }
        }
        rewards
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
