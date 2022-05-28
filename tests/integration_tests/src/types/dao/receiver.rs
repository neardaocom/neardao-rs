use serde::{Deserialize, Serialize};
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct WorkflowMessage {
    pub proposal_id: u32,
    pub storage_key: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct TreasuryMessage {
    pub partition_id: u16,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum ReceiverMessage {
    Workflow(WorkflowMessage),
    Treasury(TreasuryMessage),
}
