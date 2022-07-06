#![allow(unused)]

use std::{unimplemented, vec::Vec};

use library::functions::binding::bind_input;
use library::functions::serialization::serialize_to_json;
use library::functions::validation::validate;
use library::interpreter::expression::EExpr;
use library::storage::StorageBucket;
use library::types::Value;
use library::workflow::runtime::activity_input::UserInput;
use library::workflow::runtime::const_provider::RuntimeConstantProvider;
use library::workflow::runtime::source::SourceProvider;
use library::workflow::types::{BindDefinition, ObjectMetadata};
use library::workflow::validator::Validator;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, log, near_bindgen, require, AccountId, Balance, BorshStorageKey, PanicOnDefault,
};
use types::SourceMock;

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    Delegations,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    /// Staking contract.
    pub staking_id: AccountId,
    /// Delegations per user.
    pub delegations: LookupMap<AccountId, Balance>,
    /// Delegated token total amount.
    pub total_delegation_amount: Balance,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(staking_id: AccountId) -> Self {
        Self {
            staking_id,
            delegations: LookupMap::new(StorageKeys::Delegations),
            total_delegation_amount: 0,
        }
    }

    /// Returns total delegated stake.
    pub fn delegation_total_supply(&self) -> U128 {
        U128(self.total_delegation_amount)
    }

    pub fn get_user_weight(&self, account_id: &AccountId) -> U128 {
        U128(self.delegations.get(account_id).unwrap_or_default())
    }
}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn register_delegation(&mut self, account_id: AccountId) {
        let staking_id = self.staking_id.clone();
        require!(
            env::predecessor_account_id() == staking_id,
            "ERR_INVALID_CALLER"
        );
        self.delegations.insert(&account_id, &0);
    }

    /// Adds given amount to given account as delegated weight.
    /// Returns previous amount, new amount and total delegated amount.
    pub fn delegate_owned(
        &mut self,
        account_id: AccountId,
        amount: near_sdk::json_types::U128,
    ) -> (U128, U128, U128) {
        let staking_id = self.staking_id.clone();
        require!(
            env::predecessor_account_id() == staking_id,
            "ERR_INVALID_CALLER"
        );
        let prev_amount = self
            .delegations
            .get(&account_id)
            .expect("ERR_NOT_REGISTERED");
        let new_amount = prev_amount + amount.0;
        self.delegations.insert(&account_id, &new_amount);
        self.total_delegation_amount += amount.0;
        (
            U128(prev_amount),
            U128(new_amount),
            self.delegation_total_supply(),
        )
    }

    /// Removes given amount from given account's delegations.
    /// Returns previous, new amount of this account and total delegated amount.
    pub fn undelegate(&mut self, account_id: AccountId, amount: U128) -> (U128, U128, U128) {
        let staking_id = self.staking_id.clone();
        require!(
            env::predecessor_account_id() == staking_id,
            "ERR_INVALID_CALLER"
        );
        let prev_amount = self.delegations.get(&account_id).unwrap_or_default();
        assert!(prev_amount >= amount.0, "ERR_INVALID_STAKING_CONTRACT");
        let new_amount = prev_amount - amount.0;
        self.delegations.insert(&account_id, &new_amount);
        self.total_delegation_amount -= amount.0;
        (
            U128(prev_amount),
            U128(new_amount),
            self.delegation_total_supply(),
        )
    }

    /// Transfers amount from previous account to new account.
    /// Returns amount of transfered and total delegated amount.
    pub fn transfer_amount(
        &mut self,
        prev_account_id: AccountId,
        new_account_id: AccountId,
        amount: U128,
    ) -> (U128, U128) {
        let staking_id = self.staking_id.clone();
        require!(
            env::predecessor_account_id() == staking_id,
            "ERR_INVALID_CALLER"
        );

        let prev_amount = self
            .delegations
            .get(&prev_account_id)
            .expect("ERR_NOT_REGISTERED");

        let mut new_amount = self
            .delegations
            .get(&new_account_id)
            .expect("ERR_NOT_REGISTERED");

        let prev_amount = prev_amount
            .checked_sub(amount.0)
            .expect("Not enough tokens");
        new_amount += amount.0;

        self.delegations.insert(&prev_account_id, &prev_amount);
        self.delegations.insert(&new_account_id, &new_amount);
        log!(
            "EVENT: Transitioned amount of {} delegated tokens by {} to {}.",
            amount.0,
            prev_account_id,
            new_account_id
        );
        (amount, self.delegation_total_supply())
    }
}
