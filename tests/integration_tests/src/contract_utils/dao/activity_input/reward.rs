use std::collections::HashMap;

use library::{
    types::{activity_input::UserInput, datatype::Value},
    workflow::{
        action::{ActionInput, ActionInputType},
        types::DaoActionIdent,
    },
};

use crate::contract_utils::dao::types::reward::RewardActivity;

/// Activity inputs for `Trade1`.
pub struct ActivityInputReward1;
impl ActivityInputReward1 {
    /// Inputs to create partition with rewards.
    pub fn activity_1(
        ft_account_id: String,
        ft_amount: u64,
        ft_decimals: u64,
        near_amount: u64,
    ) -> Vec<Option<ActionInput>> {
        let mut map = HashMap::new();
        map.insert("name".into(), Value::String("partition_for_rewards".into()));
        map.insert(
            "assets.0.asset_id.ft.account_id".into(),
            Value::String(ft_account_id),
        );
        map.insert(
            "assets.0.asset_id.ft.decimals".into(),
            Value::U64(ft_decimals),
        );
        map.insert(
            "assets.0.unlocking.amount_init_unlock".into(),
            Value::U64(ft_amount),
        );
        map.insert("assets.1.asset_id.near".into(), Value::Null);
        map.insert(
            "assets.1.unlocking.amount_init_unlock".into(),
            Value::U64(near_amount),
        );
        vec![Some(ActionInput {
            action: ActionInputType::DaoAction(DaoActionIdent::TreasuryAddPartition),
            values: UserInput::Map(map),
        })]
    }
    /// Inputs to create wage reward.
    pub fn activity_2(
        wage_unit_seconds: u64,
        group_id: u64,
        role_id: u64,
        partition_id: u64,
        timestamp_from: u64,
        timestamp_to: u64,
        ft_account_id: String,
        ft_amount: u128,
        ft_decimals: u64,
        near_amount: u128,
    ) -> Vec<Option<ActionInput>> {
        let mut map = HashMap::new();
        map.insert("group_id".into(), Value::U64(group_id));
        map.insert("role_id".into(), Value::U64(role_id));
        map.insert("partition_id".into(), Value::U64(partition_id));
        map.insert("time_valid_from".into(), Value::U64(timestamp_from));
        map.insert("time_valid_to".into(), Value::U64(timestamp_to));
        map.insert(
            "type.wage.unit_seconds".into(),
            Value::U64(wage_unit_seconds),
        );
        map.insert(
            "reward_amounts.0.0.ft.account_id".into(),
            Value::String(ft_account_id),
        );
        map.insert(
            "reward_amounts.0.0.ft.decimals".into(),
            Value::U64(ft_decimals),
        );
        map.insert("reward_amounts.0.1".into(), Value::U128(ft_amount.into()));
        map.insert("reward_amounts.1.0.near".into(), Value::Null);
        map.insert("reward_amounts.1.1".into(), Value::U128(near_amount.into()));
        vec![Some(ActionInput {
            action: ActionInputType::DaoAction(DaoActionIdent::RewardAdd),
            values: UserInput::Map(map),
        })]
    }

    /// Inputs to create user activity reward.
    pub fn activity_3(
        activity_ids: Vec<RewardActivity>,
        group_id: u64,
        role_id: u64,
        partition_id: u64,
        timestamp_from: u64,
        timestamp_to: u64,
        ft_account_id: String,
        ft_amount: u128,
        ft_decimals: u64,
        near_amount: u128,
    ) -> Vec<Option<ActionInput>> {
        let mut activity_ids: Vec<u64> = activity_ids.into_iter().map(|e| e.into()).collect();
        activity_ids.sort();
        activity_ids.dedup();
        let mut map = HashMap::new();
        map.insert("group_id".into(), Value::U64(group_id));
        map.insert("role_id".into(), Value::U64(role_id));
        map.insert("partition_id".into(), Value::U64(partition_id));
        map.insert("time_valid_from".into(), Value::U64(timestamp_from));
        map.insert("time_valid_to".into(), Value::U64(timestamp_to));
        map.insert(
            "type.user_activity.activity_ids".into(),
            Value::VecU64(activity_ids),
        );
        map.insert(
            "reward_amounts.0.0.ft.account_id".into(),
            Value::String(ft_account_id),
        );
        map.insert(
            "reward_amounts.0.0.ft.decimals".into(),
            Value::U64(ft_decimals),
        );
        map.insert("reward_amounts.0.1".into(), Value::U128(ft_amount.into()));
        map.insert("reward_amounts.1.0.near".into(), Value::Null);
        map.insert("reward_amounts.1.1".into(), Value::U128(near_amount.into()));
        vec![Some(ActionInput {
            action: ActionInputType::DaoAction(DaoActionIdent::RewardAdd),
            values: UserInput::Map(map),
        })]
    }
}
