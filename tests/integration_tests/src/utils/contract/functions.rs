use crate::types::ReceiverMessage;

pub fn serialized_dao_ft_receiver_msg(proposal_id: u32) -> String {
    let msg = serde_json::to_string(&ReceiverMessage { proposal_id })
        .expect("failed to serialize dao receiver msg");
    println!("msg {}", &msg);
    msg
}
