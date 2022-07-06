use std::collections::HashMap;

use library::{
    types::Value,
    workflow::action::{ActionInput, ActionInputType},
    workflow::runtime::activity_input::UserInput,
};
use workspaces::AccountId;

/// Activity inputs for `Bounty1`.
pub struct ActivityInputBounty1;
impl ActivityInputBounty1 {
    pub fn activity_1() -> Vec<Option<ActionInput>> {
        vec![Some(ActionInput {
            action: ActionInputType::Event("event_checkin".into()),
            values: UserInput::Map(HashMap::new()),
        })]
    }
    pub fn activity_3(approved: bool) -> Vec<Option<ActionInput>> {
        let mut map = HashMap::new();
        map.insert("checkin_accepted".to_string(), Value::Bool(approved));
        vec![Some(ActionInput {
            action: ActionInputType::Event("event_approve".into()),
            values: UserInput::Map(map),
        })]
    }
    pub fn activity_4(task_result: String) -> Vec<Option<ActionInput>> {
        let mut map = HashMap::new();
        map.insert("result".to_string(), Value::String(task_result));
        vec![Some(ActionInput {
            action: ActionInputType::Event("event_done".into()),
            values: UserInput::Map(map),
        })]
    }
    pub fn activity_5(task_result_evaluation: String) -> Vec<Option<ActionInput>> {
        let mut map = HashMap::new();
        map.insert(
            "result_evaluation".to_string(),
            Value::String(task_result_evaluation),
        );
        vec![Some(ActionInput {
            action: ActionInputType::Event("event_done_approve".into()),
            values: UserInput::Map(map),
        })]
    }
    pub fn activity_6(receiver_id: &AccountId, amount_near: u128) -> Vec<Option<ActionInput>> {
        let mut map = HashMap::new();
        map.insert(
            "receiver_id".to_string(),
            Value::String(receiver_id.to_string()),
        );
        map.insert("amount_near".to_string(), Value::U128(amount_near.into()));
        vec![Some(ActionInput {
            action: ActionInputType::SendNear,
            values: UserInput::Map(map),
        })]
    }
}
