use std::collections::HashMap;

use library::{
    types::{activity_input::UserInput, datatype::Value},
    workflow::{
        action::{ActionInput, ActionInputType},
        types::DaoActionIdent,
    },
};

use crate::contract_utils::dao::types::reward::RewardTypeIdent;

/// Activity inputs for `Trade1`.
pub struct ActivityInputReward1;
impl ActivityInputReward1 {
    pub fn activity_1(
        ft_account_id: String,
        ft_amount: u128,
        ft_decimals: u64,
        near_amount: u128,
    ) -> Vec<Option<ActionInput>> {
        let mut map = HashMap::new();
        map.insert("asset_count".into(), Value::U64(2));
        map.insert("assets.0.type".into(), Value::String("ft".into()));
        map.insert("assets.0.amount".into(), Value::U128(ft_amount.into()));
        map.insert(
            "assets.0.ft_account_id".into(),
            Value::String(ft_account_id),
        );
        map.insert("assets.0.decimals".into(), Value::U64(ft_decimals));
        map.insert("assets.1.type".into(), Value::String("near".into()));
        map.insert("assets.1.amount".into(), Value::U128(near_amount.into()));
        vec![Some(ActionInput {
            action: ActionInputType::DaoAction(DaoActionIdent::TreasuryAddPartition),
            values: UserInput::Map(map),
        })]
    }
    pub fn activity_2(
        reward_name: RewardTypeIdent,
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
        let reward_name = reward_name.to_string();
        let mut map = HashMap::new();
        map.insert("group_id".into(), Value::U64(group_id));
        map.insert("role_id".into(), Value::U64(role_id));
        map.insert("partition_id".into(), Value::U64(partition_id));
        map.insert("time_valid_from".into(), Value::U64(timestamp_from));
        map.insert("time_valid_to".into(), Value::U64(timestamp_to));
        map.insert("reward_type.name".into(), Value::String(reward_name));
        map.insert(
            "reward_type.name.unit".into(),
            Value::U64(wage_unit_seconds),
        );
        map.insert("reward_count".into(), Value::U64(2));
        map.insert("reward_asset.0.type".into(), Value::String("ft".into()));
        map.insert(
            "reward_asset.0.amount".into(),
            Value::U128(ft_amount.into()),
        );
        map.insert(
            "reward_asset.0.ft_account_id".into(),
            Value::String(ft_account_id),
        );
        map.insert("reward_asset.0.decimals".into(), Value::U64(ft_decimals));
        map.insert("reward_asset.1.type".into(), Value::String("near".into()));
        map.insert(
            "reward_asset.1.amount".into(),
            Value::U128(near_amount.into()),
        );
        vec![Some(ActionInput {
            action: ActionInputType::DaoAction(DaoActionIdent::RewardAdd),
            values: UserInput::Map(map),
        })]
    }
}
