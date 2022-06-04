use std::fmt::Display;

use near_sdk::json_types::U128;
use serde::{Deserialize, Serialize};
use workspaces::AccountId;

use crate::utils::TimestampSec;

pub type TokenId = String;
pub type ApprovalId = Option<u64>;

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum Asset {
    Near,
    Ft(AssetFT),
    Nft(AssetNFT),
}

impl Asset {
    pub fn new_near() -> Self {
        Self::Near
    }
    pub fn new_ft(account_id: AccountId, decimals: u8) -> Self {
        Self::Ft(AssetFT::new(account_id, decimals))
    }
    pub fn new_nft(account_id: AccountId, token_id: String, approval_id: Option<u64>) -> Self {
        Self::Nft(AssetNFT::new(account_id, token_id, approval_id))
    }
    pub fn decimals(&self) -> u8 {
        match &self {
            Self::Near => 24,
            Self::Ft(a) => a.decimals,
            _ => 0,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct AssetFT {
    pub account_id: AccountId,
    pub decimals: u8,
}
impl AssetFT {
    pub fn new(account_id: AccountId, decimals: u8) -> Self {
        Self {
            account_id,
            decimals,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct AssetNFT {
    pub account_id: AccountId,
    pub token_id: TokenId,
    pub approval_id: ApprovalId,
}

impl AssetNFT {
    pub fn new(account_id: AccountId, token_id: TokenId, approval_id: ApprovalId) -> Self {
        Self {
            account_id,
            token_id,
            approval_id,
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

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
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
    r#type: RewardType,
    /// Defines unique asset per unit.
    reward_amounts: Vec<(Asset, u128)>,
    /// TODO: Unimplemented.
    time_valid_from: u64,
    /// TODO: Unimplemented.
    time_valid_to: u64,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum RewardType {
    /// Unit is amount of provided seconds.
    Wage(RewardWage),
    /// TODO: Implementation.
    /// Activity id of done activities: Eg. voting, staking ...
    UserActivity(RewardUserActivity),
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct Wallet {
    /// Currently provided unique rewards.
    rewards: Vec<WalletReward>,
    /// Rewards that failed to be withdrawn. These are immediately available to be withdrawn again.
    failed_withdraws: Vec<(Asset, u128)>,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct WalletReward {
    reward_id: u16,
    time_added: TimestampSec,
    /// Sums of all withdrawn assets from reward.
    /// All assets MUST be defined in the reward.
    withdraw_stats: Vec<WithdrawStats>,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum WithdrawStats {
    Wage(WageStats),
    Activity(ActivityStats),
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct RewardWage {
    /// Amount of seconds define one unit.
    pub unit_seconds: u16,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct RewardUserActivity {
    pub activity_ids: Vec<u8>,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct ActivityStats {
    pub asset_id: Asset,
    /// Count of all executions that have not been withdrawn yet.
    pub executed_count: u16,
    /// Total count of withdrawn executions.
    pub total_withdrawn_count: u16,
    pub timestamp_last_withdraw: TimestampSec,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct WageStats {
    pub asset_id: Asset,
    pub amount: u128,
    pub timestamp_last_withdraw: TimestampSec,
}

pub enum RewardActivity {
    AcceptedProposal,
    Vote,
    Delegate,
    TransitiveDelegate,
    Activity,
}

impl From<RewardActivity> for u64 {
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
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct ClaimableReward {
    pub asset: Asset,
    pub reward_id: u16,
    pub amount: U128,
    pub partition_id: u16,
}
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct ClaimableRewards {
    pub claimable_rewards: Vec<ClaimableReward>,
    pub failed_withdraws: Vec<(Asset, U128)>,
}
