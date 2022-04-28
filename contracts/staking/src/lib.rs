//! Staking service

use consts::{
    ACCOUNT_STATS_STORAGE, DAO_KEY_PREFIX, GAS_FOR_DELEGATE, GAS_FOR_FT_TRANSFER,
    GAS_FOR_UNDELEGATE, MIN_STORAGE,
};
use dao::Dao;
use library::functions::utils::into_storage_key_wrapper_u16;
use near_contract_standards::fungible_token::core_impl::ext_fungible_token;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_contract_standards::storage_management::StorageBalance;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, ext_contract, near_bindgen, require, serde_json, AccountId, Balance, BorshStorageKey,
    PanicOnDefault, Promise, PromiseOrValue, PromiseResult, StorageUsage,
};

pub use user::{User, VersionedUser};

use crate::consts::{GAS_FOR_REGISTER, MIN_REGISTER_DEPOSIT, MIN_STORAGE_FOR_DAO};

mod consts;
mod dao;
mod storage_impl;
mod user;

#[derive(BorshStorageKey, BorshSerialize)]
enum StorageKeys {
    Daos,
    StorageDeposit,
}

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct Contract {
    /// Registrar that can register new daos.
    registrar_id: AccountId,
    /// Daos using this contract.
    daos: LookupMap<AccountId, Dao>,
    /// Storage deposit amount of staked NEARs and used storage in bytes.
    dao_storage_balance: LookupMap<AccountId, AccountStats>,
    /// Suffix used for new DAOs to avoid storage keys collision.
    last_dao_key_suffix: u16,
}

#[near_bindgen]
impl Contract {
    pub fn min_storage_deposit() -> Balance {
        MIN_STORAGE as Balance * env::storage_byte_cost()
    }

    #[init]
    pub fn new(registrar_id: AccountId) -> Self {
        Self {
            registrar_id,
            daos: LookupMap::new(StorageKeys::Daos),
            dao_storage_balance: LookupMap::new(StorageKeys::StorageDeposit),
            last_dao_key_suffix: 0,
        }
    }

    /// Registers new dao in contract.
    pub fn register_new_dao(&mut self, dao_id: AccountId, vote_token_id: AccountId) {
        require!(
            env::predecessor_account_id() == self.registrar_id,
            "No rights"
        );
        let storage_before = env::storage_usage();
        let mut account_stats = self.get_account_stats(&dao_id);

        self.last_dao_key_suffix += 1;
        let key = into_storage_key_wrapper_u16(DAO_KEY_PREFIX, self.last_dao_key_suffix);
        let users = LookupMap::new(key);
        let total_amount = 0;

        let dao_struct = Dao {
            account_id: dao_id.to_owned(),
            vote_token_id,
            users,
            total_amount,
        };
        require!(
            self.daos.insert(&dao_id, &dao_struct).is_none(),
            "Dao is already registered."
        );
        let storage_after = env::storage_usage();
        let storage_diff = storage_after - storage_before;
        account_stats.add_storage_used(storage_diff);
        self.save_account_stats(&dao_id, &account_stats);
    }

    #[payable]
    /// Registers caller in dao
    /// Requires some deposit to protect dao from malicious users as it adds the storage used by dao.
    pub fn register_in_dao(&mut self, dao_id: AccountId) {
        require!(
            env::attached_deposit() >= MIN_REGISTER_DEPOSIT,
            "Not enough deposit"
        );
        let storage_before = env::storage_usage();
        let sender_id = env::predecessor_account_id();
        let mut account_stats = self.get_account_stats(&dao_id);
        let mut dao = self.get_dao(&dao_id);
        dao.register_user(&sender_id);
        self.save_dao(&dao_id, &dao);
        let storage_after = env::storage_usage();
        account_stats.add_storage_used(storage_after - storage_before);
        account_stats.inc_user_count();
        self.save_account_stats(&dao_id, &account_stats);
        account_stats.assert_enough_deposit();
        ext_dao::register_delegation(
            sender_id.clone(),
            dao_id,
            sender_id.as_bytes().len() as u128
                * MIN_STORAGE_FOR_DAO as Balance
                * env::storage_byte_cost(),
            GAS_FOR_REGISTER,
        );
    }

    #[payable]
    pub fn unregister_in_dao(&mut self, dao_id: AccountId) {
        let storage_before = env::storage_usage();
        let sender_id = env::predecessor_account_id();
        let mut account_stats = self.get_account_stats(&dao_id);
        let mut dao = self.get_dao(&dao_id);
        dao.unregister_user(&sender_id);
        self.save_dao(&dao_id, &dao);
        let storage_after = env::storage_usage();
        account_stats.remove_storage_used(storage_before - storage_after);
        account_stats.dec_user_count();
        self.save_account_stats(&dao_id, &account_stats);
        account_stats.assert_enough_deposit();
    }

    /// Delegates owned tokens.
    pub fn delegate_owned(
        &mut self,
        dao_id: AccountId,
        delegate_id: AccountId,
        amount: U128,
    ) -> Promise {
        let storage_before = env::storage_usage();
        let sender_id = env::predecessor_account_id();
        let mut dao = self.get_dao(&dao_id);
        dao.delegate_owned(sender_id.clone(), delegate_id.clone(), amount.0);
        let mut account_stats = self.get_account_stats(&dao_id);
        self.save_dao(&dao_id, &dao);
        let storage_after = env::storage_usage();
        account_stats.add_storage_used(storage_after - storage_before);
        self.save_account_stats(&dao_id, &account_stats);
        account_stats.assert_enough_deposit();
        ext_dao::delegate_owned(delegate_id, amount, dao.account_id, 0, GAS_FOR_DELEGATE)
    }
    /// Undelegates tokens.
    pub fn undelegate(
        &mut self,
        dao_id: AccountId,
        delegate_id: AccountId,
        amount: U128,
    ) -> Promise {
        let storage_before = env::storage_usage();
        let sender_id = env::predecessor_account_id();
        let mut dao = self.get_dao(&dao_id);
        dao.undelegate(sender_id.clone(), delegate_id.clone(), amount.0);
        self.save_dao(&dao_id, &dao);
        let storage_after = env::storage_usage();
        let mut account_stats = self.get_account_stats(&dao_id);
        account_stats.remove_storage_used(storage_before - storage_after);
        self.save_account_stats(&dao_id, &account_stats);
        account_stats.assert_enough_deposit();
        ext_dao::undelegate(delegate_id, amount, dao.account_id, 0, GAS_FOR_UNDELEGATE)
    }
    // TODO: Figure out storage management - maybe just let it be this way.
    /// Delegate all delegated tokens to delegate.
    pub fn delegate(&mut self, dao_id: AccountId, delegate_id: AccountId) -> Promise {
        let storage_before = env::storage_usage();
        let sender_id = env::predecessor_account_id();
        let mut account_stats = self.get_account_stats(&dao_id);
        let mut dao = self.get_dao(&dao_id);
        let amount = dao.delegate(&sender_id, delegate_id.clone());
        self.save_dao(&dao_id, &dao);
        let storage_after = env::storage_usage();
        if storage_after >= storage_before {
            account_stats.add_storage_used(storage_after - storage_before);
        } else {
            account_stats.remove_storage_used(storage_before - storage_after);
        }
        self.save_account_stats(&dao_id, &account_stats);
        account_stats.assert_enough_deposit();
        ext_dao::transfer_amount(
            sender_id,
            delegate_id,
            amount.into(),
            dao.account_id.clone(),
            0,
            GAS_FOR_UNDELEGATE,
        )
    }
    /// Withdraw staked tokens.
    pub fn withdraw(&mut self, dao_id: AccountId, amount: U128) -> Promise {
        let sender_id = env::predecessor_account_id();
        let mut dao = self.get_dao(&dao_id);
        dao.user_withdraw(&sender_id, amount.0);
        self.save_dao(&dao_id, &dao);
        ext_fungible_token::ft_transfer(
            sender_id.clone(),
            amount,
            None,
            dao.vote_token_id.clone(),
            1,
            GAS_FOR_FT_TRANSFER,
        )
        .then(ext_self::exchange_callback_post_withdraw(
            dao_id,
            sender_id,
            amount,
            env::current_account_id(),
            0,
            GAS_FOR_FT_TRANSFER,
        ))
    }
    /// Checks if withdraw was succesful.
    /// Reverts changes if not.
    #[private]
    pub fn exchange_callback_post_withdraw(
        &mut self,
        dao_id: AccountId,
        sender_id: AccountId,
        amount: U128,
    ) {
        require!(
            env::promise_results_count() == 1,
            "ERR_CALLBACK_POST_WITHDRAW_INVALID",
        );
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {}
            PromiseResult::Failed => {
                let mut dao = self.get_dao(&dao_id);
                // This reverts the changes from withdraw function.
                dao.user_deposit(sender_id, amount.0);
                self.save_dao(&dao_id, &dao);
            }
        };
    }
    /// Total staked amount in dao.
    pub fn dao_ft_total_supply(&self, dao_id: AccountId) -> U128 {
        let dao = self.get_dao(&dao_id);
        dao.ft_total_supply()
    }
    /// Total number of tokens staked by given user in dao.
    pub fn dao_ft_balance_of(&self, dao_id: AccountId, account_id: AccountId) -> U128 {
        let dao = self.get_dao(&dao_id);
        dao.ft_balance_of(account_id)
    }
    /// Returns user information.
    pub fn dao_get_user(&self, dao_id: AccountId, account_id: AccountId) -> User {
        let dao = self.get_dao(&dao_id);
        dao.get_user(&account_id)
    }
}

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let dao_transfer: TransferMsgInfo =
            serde_json::from_str(msg.as_str()).expect("Missing dao info");

        let mut dao = self.get_dao(&dao_transfer.dao_id);

        require!(
            dao.vote_token_id == env::predecessor_account_id(),
            "Invalid token"
        );

        dao.user_deposit(sender_id, amount.0);
        self.save_dao(&dao_transfer.dao_id, &dao);
        PromiseOrValue::Value(U128(0))
    }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
struct TransferMsgInfo {
    pub dao_id: AccountId,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct AccountStats {
    near_amount: Balance,
    storage_used: StorageUsage,
    users_registered: Balance,
}

impl AccountStats {
    pub fn new(near_amount: Balance, storage_used: StorageUsage) -> Self {
        Self {
            near_amount,
            storage_used,
            users_registered: 0,
        }
    }
    pub fn total_balance(&self) -> Balance {
        self.near_amount
    }
    pub fn available_balance(&self) -> Balance {
        self.near_amount - self.storage_used as Balance * env::storage_byte_cost()
    }
    pub fn storage_balance(&self) -> StorageBalance {
        StorageBalance {
            total: self.near_amount.into(),
            available: self.available_balance().into(),
        }
    }
    pub fn users_registered(&self) -> Balance {
        self.users_registered
    }
    pub fn add_storage_used(&mut self, amount: StorageUsage) {
        self.storage_used += amount;
    }
    pub fn remove_storage_used(&mut self, amount: StorageUsage) {
        self.storage_used = self
            .storage_used
            .checked_sub(amount)
            .expect("Internal error");
    }
    pub fn add_balance(&mut self, amount: Balance) {
        self.near_amount += amount;
    }
    pub fn remove_balance(&mut self, amount: Balance) {
        self.near_amount = self
            .near_amount
            .checked_sub(amount)
            .expect("Cannot remove more balance than available");
    }
    pub fn inc_user_count(&mut self) {
        self.users_registered += 1;
    }
    pub fn dec_user_count(&mut self) {
        self.users_registered -= 1;
    }
    pub fn assert_enough_deposit(&self) {
        require!(
            self.storage_used as Balance * env::storage_byte_cost() <= self.near_amount,
            "Dao does not have enough deposit."
        )
    }
}

#[ext_contract(ext_dao)]
pub trait Contract {
    fn register_delegation(&mut self, account_id: AccountId);
    fn delegate_owned(&mut self, account_id: AccountId, amount: U128);
    fn undelegate(&mut self, account_id: AccountId, amount: U128);
    fn transfer_amount(
        &mut self,
        prev_account_id: AccountId,
        new_account_id: AccountId,
        amount: U128,
    );
}

#[ext_contract(ext_self)]
pub trait Contract {
    fn exchange_callback_post_withdraw(
        &mut self,
        dao_id: AccountId,
        sender_id: AccountId,
        amount: U128,
    );
}

impl Contract {
    pub fn get_dao(&self, dao_id: &AccountId) -> Dao {
        self.daos.get(dao_id).expect("Dao not found")
    }
    pub fn save_dao(&mut self, dao_id: &AccountId, dao: &Dao) -> Option<Dao> {
        self.daos.insert(dao_id, &dao)
    }
    fn register_account(&mut self, account_id: &AccountId, amount: Balance) {
        let storage = AccountStats::new(
            amount,
            ACCOUNT_STATS_STORAGE + account_id.as_bytes().len() as StorageUsage,
        );
        self.dao_storage_balance.insert(account_id, &storage);
    }
    pub fn get_account_stats(&self, account_id: &AccountId) -> AccountStats {
        self.dao_storage_balance
            .get(account_id)
            .expect("Account not registered")
    }
    pub fn save_account_stats(&mut self, account_id: &AccountId, stats: &AccountStats) {
        self.dao_storage_balance.insert(account_id, stats);
    }
}

#[cfg(test)]
mod tests {
    /* use near_contract_standards::storage_management::StorageManagement;
    use near_sdk::json_types::U64;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;

    use near_sdk_sim::to_yocto;

    use super::*;

    #[test]
    fn test_basics() {
        const UNSTAKE_PERIOD: u64 = 1000;
        let contract_owner: AccountId = accounts(0);
        let voting_token: AccountId = accounts(1);
        let delegate_from_user: AccountId = accounts(2);
        let delegate_to_user: AccountId = accounts(3);

        let mut context = VMContextBuilder::new();

        testing_env!(context
            .predecessor_account_id(contract_owner.clone())
            .build());
        let mut contract = Contract::new(contract_owner, voting_token.clone(), U64(UNSTAKE_PERIOD));

        testing_env!(context.attached_deposit(to_yocto("1")).build());
        contract.storage_deposit(Some(delegate_from_user.clone()), None);

        testing_env!(context.predecessor_account_id(voting_token.clone()).build());
        contract.ft_on_transfer(
            delegate_from_user.clone(),
            U128(to_yocto("100")),
            "".to_string(),
        );
        assert_eq!(contract.ft_total_supply().0, to_yocto("100"));
        assert_eq!(
            contract.ft_balance_of(delegate_from_user.clone()).0,
            to_yocto("100")
        );

        testing_env!(context
            .predecessor_account_id(delegate_from_user.clone())
            .build());
        contract.withdraw(U128(to_yocto("50")));
        assert_eq!(contract.ft_total_supply().0, to_yocto("50"));
        assert_eq!(
            contract.ft_balance_of(delegate_from_user.clone()).0,
            to_yocto("50")
        );

        testing_env!(context.attached_deposit(to_yocto("1")).build());
        contract.storage_deposit(Some(delegate_to_user.clone()), None);

        contract.delegate(delegate_to_user.clone(), U128(to_yocto("10")));
        let user = contract.get_user(delegate_from_user.clone());
        assert_eq!(user.delegated_amount(), to_yocto("10"));

        contract.undelegate(delegate_to_user, U128(to_yocto("10")));
        let user = contract.get_user(delegate_from_user);
        assert_eq!(user.delegated_amount(), 0);
        assert_eq!(user.next_action_timestamp, U64(UNSTAKE_PERIOD));
    } */
}
