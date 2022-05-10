use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::Serialize;
use near_sdk::AccountId;

use crate::core::*;
use crate::internal::utils::current_timestamp_sec;
use crate::{
    treasury::{Asset, TreasuryPartition},
    TimestampSec,
};
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
    /// Return amount of total available asset reward.
    pub fn available_asset_amount(&self, asset: &Asset, current_timestamp: TimestampSec) -> u128 {
        let amount = if let Some((_, amount)) = self.reward_amounts.iter().find(|(r, _)| r == asset)
        {
            amount
        } else {
            return 0;
        };
        let amount = match self.r#type {
            RewardType::Wage(seconds) => {
                let seconds_passed = current_timestamp - self.time_valid_from;
                let units = seconds_passed / seconds as u64;
                amount * units as u128
            }
            RewardType::UserActivity(_) => todo!(),
        };
        amount.into()
    }
}

/// TODO: Refactor from tuples into structs.
#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub enum RewardType {
    /// Unit is amount of provided seconds.
    Wage(u16),
    /// TODO: Implementation.
    /// Activity id of done activities: Eg. voting, staking ...
    UserActivity(Vec<u8>),
}

impl RewardType {
    pub fn new_wage(unit_seconds: u16) -> Self {
        RewardType::Wage(unit_seconds)
    }
    pub fn new_user_activity(activity_ids: Vec<u8>) -> Self {
        RewardType::UserActivity(activity_ids)
    }
}

impl Contract {
    /// TODO: Error handling and validations.
    /// Adds new reward entry into DAO rewards.
    /// Also adds this reward in affected user wallets.
    pub fn add_reward(&mut self, reward: &Reward) -> u16 {
        let partition = self
            .treasury_partition
            .get(&reward.partition_id)
            .expect("partition not found");

        assert!(
            self.validate_reward_assets(reward, &partition),
            "partion does not have all required assets"
        );

        let group = self.groups.get(&reward.group_id).expect("group not found");
        let rewarded_users =
            self.get_group_members_with_role(reward.group_id, &group, reward.role_id);

        self.reward_last_id += 1;

        // Add reward to role members wallets.
        let reward_assets: Vec<Asset> = reward
            .reward_amounts()
            .into_iter()
            .map(|(a, _)| a.to_owned())
            .collect();
        let current_timestamp = current_timestamp_sec();
        for user in rewarded_users {
            assert!(
                self.add_reward_to_user_wallet(
                    self.reward_last_id,
                    &user,
                    reward_assets.clone(),
                    current_timestamp
                ),
                "failed to added reward to user wallet"
            );
        }
        self.rewards.insert(&self.reward_last_id, reward);
        self.reward_last_id
    }
    /// TODO: Check it can be safely removed.
    pub fn remove_reward(&mut self, reward_id: u16) -> Option<Reward> {
        self.rewards.remove(&reward_id)
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
        account_id: &AccountId,
        assets: Vec<Asset>,
        current_timestamp: TimestampSec,
    ) -> bool {
        let mut wallet = self.get_wallet(account_id);
        let added = wallet.add_reward(reward_id, current_timestamp, assets);
        self.wallets.insert(account_id, &wallet);
        added
    }
}
