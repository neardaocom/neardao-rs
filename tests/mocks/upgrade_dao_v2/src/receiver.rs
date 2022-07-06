use crate::core::*;
use library::types::Value;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_contract_standards::non_fungible_token::approval::NonFungibleTokenApprovalReceiver;
use near_contract_standards::non_fungible_token::core::NonFungibleTokenReceiver;
use near_sdk::json_types::U128;
use near_sdk::serde::Deserialize;
use near_sdk::{env, near_bindgen, serde_json, AccountId, PromiseOrValue};

#[derive(Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ReceiverMessage {
    pub proposal_id: u32,
}

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    /// TODO: Implement.
    /// TODO: Figure out how to assign storage keys.
    /// Required for some workflow scenarios.
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let msg: ReceiverMessage = serde_json::from_str(&msg).expect("invalid receiver msg");
        let prop_settings = self
            .workflow_propose_settings
            .get(&msg.proposal_id)
            .expect("proposal id does not exist");
        let storage_key = prop_settings
            .storage_key
            .expect("workflow does not have storage");
        let mut storage = self.storage.get(&storage_key).unwrap();
        storage.add_data(
            &"sender_id".to_string(),
            &Value::String(sender_id.to_string()),
        );
        storage.add_data(
            &"token_id".to_string(),
            &Value::String(env::predecessor_account_id().to_string()),
        );
        storage.add_data(&"amount".to_string(), &Value::U128(amount));
        self.storage.insert(&storage_key, &storage);
        PromiseOrValue::Value(U128(0))
    }
}

#[near_bindgen]
impl NonFungibleTokenReceiver for Contract {
    fn nft_on_transfer(
        &mut self,
        sender_id: AccountId,
        previous_owner_id: AccountId,
        token_id: near_contract_standards::non_fungible_token::TokenId,
        msg: String,
    ) -> PromiseOrValue<bool> {
        todo!()
    }
}

#[near_bindgen]
impl NonFungibleTokenApprovalReceiver for Contract {
    fn nft_on_approve(
        &mut self,
        token_id: near_contract_standards::non_fungible_token::TokenId,
        owner_id: AccountId,
        approval_id: u64,
        msg: String,
    ) -> near_sdk::PromiseOrValue<String> {
        todo!()
    }
}
