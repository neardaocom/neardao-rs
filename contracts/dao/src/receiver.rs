use crate::contract::*;
use crate::proposal::Proposal;
use crate::treasury::{Asset, TreasuryPartition};
use library::types::datatype::Value;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::json_types::U128;
use near_sdk::serde::Deserialize;
use near_sdk::{env, log, near_bindgen, serde_json, AccountId, PromiseOrValue};

#[derive(Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct WorkflowMessage {
    pub proposal_id: u32,
    pub storage_key: String,
}

#[derive(Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TreasuryMessage {
    pub partition_id: u16,
}

#[derive(Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum ReceiverMessage {
    Workflow(WorkflowMessage),
    Treasury(TreasuryMessage),
}

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let msg: ReceiverMessage = serde_json::from_str(&msg).expect("Invalid receiver msg");
        match msg {
            ReceiverMessage::Workflow(msg) => {
                if let Some(proposal) = self.proposals.get(&msg.proposal_id) {
                    let proposal: Proposal = proposal.into();
                    if let Some((tpl, _)) = self.workflow_template.get(&proposal.workflow_id) {
                        let keys = tpl.receiver_storage_keys;
                        if let Some(receiver) = keys.into_iter().find(|k| *k.id == msg.storage_key)
                        {
                            let prop_settings = self
                                .workflow_propose_settings
                                .get(&msg.proposal_id)
                                .unwrap();
                            let storage_key = prop_settings.storage_key.unwrap();
                            let mut storage = self.storage.get(&storage_key).unwrap();
                            if let Some(value) = storage.get_data(&receiver.amount) {
                                let mut storage_amount = value
                                    .try_into_u128()
                                    .expect("Invalid value stored in storage");
                                storage_amount += amount.0;
                                storage
                                    .add_data(&receiver.amount, &Value::U128(U128(storage_amount)));
                            } else {
                                storage.add_data(&receiver.amount, &Value::U128(U128(amount.0)));
                                storage.add_data(
                                    &receiver.token_id,
                                    &Value::String(env::predecessor_account_id().to_string()),
                                );
                                storage.add_data(
                                    &receiver.sender_id,
                                    &Value::String(sender_id.to_string()),
                                );
                            }
                            self.storage.insert(&storage_key, &storage);
                        }
                    }
                }
            }
            ReceiverMessage::Treasury(msg) => {
                if let Some(partition) = self.treasury_partition.get(&msg.partition_id) {
                    let mut partition: TreasuryPartition = partition.into();
                    let asset = Asset::new_ft(env::predecessor_account_id(), 24);
                    let mut found = false;
                    let mut id = 0;
                    while let Some(ref curr_asset) = self.cache_assets.get(&id) {
                        if curr_asset == &asset {
                            found = true;
                            break;
                        }
                        id += 1;
                    }
                    if found {
                        partition.add_amount(id, amount.0);
                        self.treasury_partition
                            .insert(&msg.partition_id, &partition.into());
                    } else {
                        log!("Asset FT not found in the asset registry.");
                    }
                }
            }
        }
        PromiseOrValue::Value(U128(0))
    }
}
