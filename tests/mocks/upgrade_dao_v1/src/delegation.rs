//! Delegations API for DAO.
//! Only staking contract is allowed to call methods in this module.
//! Forked and modified code from https://github.com/near-daos/sputnik-dao-contract/blob/main/sputnikdao2/src/delegation.rs

use crate::{core::*, reward::RewardActivity, settings::Settings};
use near_sdk::{env, json_types::U128, log, near_bindgen, require, AccountId};

impl Contract {
    pub fn update_token_holders_count(&mut self, previous_amount: u128, new_amount: u128) {
        if previous_amount > 0 && new_amount == 0 {
            self.total_delegators_count -= 1;
        } else if previous_amount == 0 && new_amount > 0 {
            self.total_delegators_count += 1;
        }
    }
}

const ERR_NOT_REGISTERED: &str = "account not registered";
const ERR_CALLER: &str = "invalid caller";
const ERR_STAKING_INTERNAL: &str = "staking internal";

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn register_delegation(&mut self, account_id: AccountId) {
        let settings: Settings = self.settings.get().unwrap().into();
        require!(
            env::predecessor_account_id() == settings.staking_id,
            ERR_CALLER
        );
        log!("register_delegation for account: {}", &account_id);
        self.delegations.insert(&account_id, &0);
    }
    /// Adds given amount to given account as delegated weight.
    /// Returns previous amount, new amount and total delegated amount.
    pub fn delegate_owned(&mut self, account_id: AccountId, amount: U128) -> (U128, U128, U128) {
        let settings: Settings = self.settings.get().unwrap().into();
        require!(
            env::predecessor_account_id() == settings.staking_id,
            ERR_CALLER
        );
        let prev_amount = self.delegations.get(&account_id).expect(ERR_NOT_REGISTERED);
        let new_amount = prev_amount + amount.0;
        self.delegations.insert(&account_id, &new_amount);
        self.update_token_holders_count(prev_amount, new_amount);
        self.total_delegation_amount += amount.0;
        self.register_executed_activity(&account_id, RewardActivity::Delegate.into());
        (
            U128(prev_amount),
            U128(new_amount),
            self.delegation_total_supply(),
        )
    }
    /// Removes given amount from given account's delegations.
    /// Returns previous, new amount of this account and total delegated amount.
    pub fn undelegate(&mut self, account_id: AccountId, amount: U128) -> (U128, U128, U128) {
        let settings: Settings = self.settings.get().unwrap().into();
        require!(
            env::predecessor_account_id() == settings.staking_id,
            ERR_CALLER
        );
        let prev_amount = self.delegations.get(&account_id).unwrap_or_default();
        require!(prev_amount >= amount.0, ERR_STAKING_INTERNAL);
        let new_amount = prev_amount - amount.0;
        self.update_token_holders_count(prev_amount, new_amount);
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
        let settings: Settings = self.settings.get().unwrap().into();
        require!(
            env::predecessor_account_id() == settings.staking_id,
            ERR_CALLER
        );
        let prev_account_prev_amount = self
            .delegations
            .get(&prev_account_id)
            .expect(ERR_NOT_REGISTERED);
        let new_account_prev_amount = self
            .delegations
            .get(&new_account_id)
            .expect(ERR_NOT_REGISTERED);
        let prev_account_new_amount = prev_account_prev_amount
            .checked_sub(amount.0)
            .expect(ERR_STAKING_INTERNAL);
        let new_account_new_amount = new_account_prev_amount + amount.0;
        if new_account_prev_amount == 0 && new_account_new_amount > 0 {
            self.total_delegators_count += 1;
        }
        if prev_account_prev_amount > 0 && prev_account_new_amount == 0 {
            self.total_delegators_count -= 1;
        }
        self.delegations
            .insert(&prev_account_id, &prev_account_new_amount);
        self.delegations
            .insert(&new_account_id, &new_account_new_amount);
        log!(
            "EVENT: Transitioned amount of {} delegated tokens by {} to {}.",
            amount.0,
            prev_account_id,
            new_account_id
        );
        self.register_executed_activity(
            &prev_account_id,
            RewardActivity::TransitiveDelegate.into(),
        );
        (amount, self.delegation_total_supply())
    }
}
