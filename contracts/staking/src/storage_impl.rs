use std::convert::TryInto;

use near_contract_standards::storage_management::{
    StorageBalance, StorageBalanceBounds, StorageManagement,
};
use near_sdk::{assert_one_yocto, env::panic_str, require};

use crate::*;

/// Implements users storage management for the pool.
#[near_bindgen]
impl StorageManagement for Contract {
    #[payable]
    fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance {
        let deposit_amount = env::attached_deposit();
        let account_id = account_id.unwrap_or_else(env::predecessor_account_id);

        let min_balance = Contract::min_storage_deposit();
        if deposit_amount < min_balance {
            env::panic_str("ERR_DEPOSIT_LESS_THAN_MIN_STORAGE");
        }

        let registration_only = registration_only.unwrap_or(false);
        if registration_only {
            self.register_account(&account_id, min_balance);
            let refund = deposit_amount - min_balance;
            if refund > 0 {
                Promise::new(env::predecessor_account_id()).transfer(refund);
            }
        } else {
            self.register_account(&account_id, deposit_amount);
        }
        self.storage_balance_of(account_id.try_into().unwrap())
            .unwrap()
    }

    #[payable]
    fn storage_withdraw(&mut self, amount: Option<U128>) -> StorageBalance {
        assert_one_yocto();
        let account_id = env::predecessor_account_id();
        let mut account_stats = self
            .dao_storage_balance
            .get(&account_id)
            .expect("Not registered");
        let available_amount = account_stats.available_balance();
        let withdraw_amount = amount.map(|a| a.0).unwrap_or(available_amount);
        require!(
            withdraw_amount <= available_amount,
            "Cannot withdraw more than available"
        );
        Promise::new(account_id.clone()).transfer(withdraw_amount);
        account_stats.remove_balance(withdraw_amount);
        self.dao_storage_balance.insert(&account_id, &account_stats);
        account_stats.storage_balance()
    }

    #[allow(unused_variables)]
    #[payable]
    fn storage_unregister(&mut self, force: Option<bool>) -> bool {
        assert_one_yocto();
        if let Some(true) = force {
            // TODO: figure out force option logic.
            panic_str("Force option is not currently supported");
        }
        let account_id = env::predecessor_account_id();
        let account_stats = self.dao_storage_balance.get(&account_id);
        let result = if let Some(stats) = account_stats {
            require!(
                stats.users_registered() == 0,
                "Non-zero amount of registered users"
            );
            self.dao_storage_balance.remove(&account_id);
            Promise::new(account_id.clone()).transfer(stats.total_balance());
            true
        } else {
            false
        };
        result
    }

    fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        StorageBalanceBounds {
            min: U128(Contract::min_storage_deposit()),
            max: None,
        }
    }

    fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBalance> {
        self.dao_storage_balance
            .get(&account_id)
            .map(|storage| storage.storage_balance())
    }
}
