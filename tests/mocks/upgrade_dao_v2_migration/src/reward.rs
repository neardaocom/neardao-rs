use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

use near_sdk::serde::Serialize;

use crate::internal::utils::current_timestamp_sec;
use crate::wallet::Wallet;
use crate::{core::*, derive_from_versioned, derive_into_versioned, RewardId, RoleId};
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

#[derive(BorshDeserialize, BorshSerialize, Serialize, Clone)]
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

#[derive(BorshDeserialize, BorshSerialize, Serialize, Clone)]
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
#[serde(crate = "near_sdk::serde")]
pub struct RewardWage {
    /// Amount of seconds define one unit.
    /// Must be > 0.
    pub unit_seconds: u16,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct RewardUserActivity {
    pub activity_ids: Vec<u8>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum RewardTypeIdent {
    Wage,
    UserActivity,
}

pub enum RewardActivity {
    AcceptedProposal,
    Vote,
    Delegate,
    TransitiveDelegate,
    Activity,
}
