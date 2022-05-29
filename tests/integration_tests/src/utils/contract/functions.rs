use crate::types::{ReceiverMessage, TreasuryMessage, WorkflowMessage};

pub fn serialized_dao_ft_receiver_workflow_msg(proposal_id: u32, id: &str) -> String {
    let msg = serde_json::to_string(&ReceiverMessage::Workflow(WorkflowMessage {
        proposal_id,
        storage_key: id.to_string(),
    }))
    .expect("failed to serialize dao receiver workflow msg");
    msg
}

pub fn serialized_dao_ft_receiver_treasury_msg(partition_id: u16) -> String {
    let msg = serde_json::to_string(&ReceiverMessage::Treasury(TreasuryMessage { partition_id }))
        .expect("failed to serialize dao receiver treasury msg");
    msg
}
