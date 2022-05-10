use std::fmt::Display;

use serde::{Deserialize, Serialize};
use workspaces::AccountId;

use crate::utils::TimestampSec;

pub type TokenId = String;
pub type ApprovalId = Option<u64>;

#[derive(Deserialize, Serialize, Clone, Debug)]
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
    pub fn new_ft(account_id: String, decimals: u8) -> Self {
        Self::FT(AccountId::try_from(account_id).unwrap(), decimals)
    }
    pub fn new_nft(account_id: String, token_id: String, approval_id: Option<u64>) -> Self {
        Self::NFT(
            AccountId::try_from(account_id).unwrap(),
            token_id,
            approval_id,
        )
    }
    pub fn decimals(&self) -> u8 {
        match &self {
            Self::Near => 24,
            Self::FT(_, decimals) => *decimals,
            _ => 0,
        }
    }
}

pub enum RewardTypeIdent {
    Wage,
    Activity,
}

impl Display for RewardTypeIdent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RewardTypeIdent::Wage => write!(f, "wage"),
            RewardTypeIdent::Activity => write!(f, "activity"),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
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

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum RewardType {
    /// Unit is amount of provided seconds.
    Wage(u16),
    /// TODO: Implementation.
    /// Activity id of done activities: Eg. voting, staking ...
    UserActivity(Vec<u8>),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Wallet {
    /// Currently provided unique rewards.
    rewards: Vec<WalletReward>,
    /// Rewards that failed to be withdrawn. These are immediately available to be withdrawn again.
    failed_withdraws: Vec<(Asset, u128)>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct WalletReward {
    reward_id: u16,
    time_added: TimestampSec,
    /// Sums of all withdrawn assets from reward.
    /// All assets MUST be defined in the reward.
    withdraw_stats: Vec<WithdrawStats>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct WithdrawStats {
    pub asset_id: Asset,
    pub amount: u128,
}
