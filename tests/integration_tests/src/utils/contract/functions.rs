use crate::types::{ReceiverMessage, WorkflowMessage};

pub fn serialized_dao_ft_receiver_msg(proposal_id: u32, id: &str) -> String {
    let msg = serde_json::to_string(&ReceiverMessage::Workflow(WorkflowMessage {
        proposal_id,
        storage_key: id.to_string(),
    }))
    .expect("failed to serialize dao receiver msg");
    println!("msg {}", &msg);
    msg
}
