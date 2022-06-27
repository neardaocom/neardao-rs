use std::collections::HashMap;

use library::{
    types::{activity_input::UserInput, datatype::Value},
    workflow::{
        action::{ActionInput, ActionInputType},
        types::DaoActionIdent,
    },
};

/// Activity inputs for `Lock1`.
pub struct ActivityInputLock1;
impl ActivityInputLock1 {
    pub fn activity_1() -> Vec<Option<ActionInput>> {
        let map = HashMap::new();
        vec![Some(ActionInput {
            action: ActionInputType::DaoAction(DaoActionIdent::TreasuryAddPartition),
            values: UserInput::Map(map),
        })]
    }
    pub fn propose_settings_activity_1(asset_ft_name: &str) -> HashMap<String, Value> {
        let mut map = HashMap::new();
        map.insert(
            "name".into(),
            Value::String("treasury partition name".into()),
        );
        map.insert("assets.0.asset_id.near".into(), Value::Null);
        map.insert(
            "assets.0.unlocking.amount_init_unlock".into(),
            Value::U64(0),
        );
        map.insert(
            "assets.0.unlocking.lock.amount_total_lock".into(),
            Value::U64(1000),
        );
        map.insert(
            "assets.0.unlocking.lock.start_from".into(),
            Value::U64(1653458400),
        );
        map.insert(
            "assets.0.unlocking.lock.duration".into(),
            Value::U64(2 * 604800),
        );
        map.insert(
            "assets.0.unlocking.lock.periods.0.type".into(),
            Value::String("linear".into()),
        );
        map.insert(
            "assets.0.unlocking.lock.periods.0.duration".into(),
            Value::U64(2 * 604800),
        );
        map.insert(
            "assets.0.unlocking.lock.periods.0.amount".into(),
            Value::U64(1000),
        );
        map.insert(
            "assets.1.asset_id.ft.account_id".into(),
            Value::String(asset_ft_name.into()),
        );
        map.insert("assets.1.asset_id.ft.decimals".into(), Value::U64(24));
        map.insert(
            "assets.1.unlocking.amount_init_unlock".into(),
            Value::U64(420),
        );
        map
    }

    pub fn activity_2_add_ft(
        partition_id: u64,
        ft_token_id: u8,
        amount: u128,
    ) -> Vec<Option<ActionInput>> {
        let mut map = HashMap::new();
        map.insert("id".into(), Value::U64(partition_id));
        map.insert("asset_id".into(), Value::U64(ft_token_id as u64));
        map.insert("amount".into(), Value::U128(amount.into()));
        vec![Some(ActionInput {
            action: ActionInputType::DaoAction(DaoActionIdent::PartitionAddAssetAmount),
            values: UserInput::Map(map),
        })]
    }
}
