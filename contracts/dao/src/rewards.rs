use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

use crate::{core::Contract, treasury::Asset, TimestampSec};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Reward {
    /// Partition from which the assets are taken.
    /// Partition must have all defined assets.
    partition_id: u16,
    /// Defines reward asset unit:
    /// - for `RewardType::Wage(seconds)` the unit is time.
    /// - for `RewardType::UserActivity(activity_ids)` the unit is per activity done.
    r#type: RewardType,
    /// Defines asset per unit.
    reward_amounts: Vec<(Asset, u128)>,
    /// TODO: Unimplemented.
    time_valid_from: u64,
    /// TODO: Unimplemented.
    time_valid_to: u64,
}

impl Reward {
    pub fn new(
        partition_id: u16,
        r#type: RewardType,
        reward_amounts: Vec<(Asset, u128)>,
        time_valid_from: TimestampSec,
        time_valid_to: TimestampSec,
    ) -> Self {
        Self {
            partition_id,
            r#type,
            reward_amounts,
            time_valid_from,
            time_valid_to,
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub enum RewardType {
    /// Unit is amount of provided seconds.
    Wage(u8),
    /// TODO: Implementation.
    /// Activity id of done activities: Eg. voting, staking ...
    UserActivity(Vec<u8>),
}

impl RewardType {
    pub fn new_wage(unit_seconds: u8) -> Self {
        RewardType::Wage(unit_seconds)
    }
    pub fn new_user_activity(activity_ids: Vec<u8>) -> Self {
        RewardType::UserActivity(activity_ids)
    }
}

impl Contract {
    pub fn add_reward(&mut self, reward: &Reward) -> u16 {
        self.reward_last_id += 1;
        self.rewards.insert(&self.reward_last_id, reward);
        self.reward_last_id
    }
    // TODO: Check it can be removed.
    pub fn remove_reward(&mut self, reward_id: u16) -> Option<Reward> {
        self.rewards.remove(&reward_id)
    }
}
