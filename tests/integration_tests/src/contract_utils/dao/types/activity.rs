use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct ReceiverMessage {
    pub proposal_id: u32,
}
