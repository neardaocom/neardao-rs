use library::types::Value;
use serde::{Deserialize, Serialize};
use workspaces::AccountId;

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct ActionLog {
    pub caller: AccountId,
    pub activity_id: u8,
    pub action_id: u8,
    pub timestamp_sec: u64,
    pub user_inputs: Vec<(String, Value)>,
}
