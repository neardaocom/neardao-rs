//! Staking service

use consts::{
    ACCOUNT_STATS_STORAGE, DAO_KEY_PREFIX, GAS_FOR_DELEGATE, GAS_FOR_FT_TRANSFER,
    GAS_FOR_UNDELEGATE, MIN_STORAGE,
};
use dao::Dao;
use library::functions::utils::into_storage_key_wrapper_u16;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_contract_standards::storage_management::StorageBalance;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, ext_contract, near_bindgen, require, serde_json, AccountId, Balance, BorshStorageKey, Gas,
    PanicOnDefault, Promise, PromiseOrValue, PromiseResult, StorageUsage,
};

pub use user::{User, VersionedUser};

use crate::consts::{
    FT_STORAGE_DEPOSIT, GAS_FOR_REGISTER, MIN_REGISTER_DEPOSIT, MIN_STORAGE_FOR_DAO,
};

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
    /// Daos using this contract.
    daos: LookupMap<AccountId, Dao>,
    /// Storage deposit amount of staked NEARs and used storage in bytes.
    dao_storage_balance: LookupMap<AccountId, AccountStats>,
    /// Sequence suffix used for new DAOs internal lookupmap to avoid storage keys collision.
    last_dao_key_suffix: u16,
}

#[near_bindgen]
impl Contract {
    pub fn min_storage_deposit() -> Balance {
        MIN_STORAGE as Balance * env::storage_byte_cost()
    }

    #[init]
    pub fn new() -> Self {
        Self {
            daos: LookupMap::new(StorageKeys::Daos),
            dao_storage_balance: LookupMap::new(StorageKeys::StorageDeposit),
            last_dao_key_suffix: 0,
        }
    }

    /// Registers new dao in contract.
    /// Dao must have done storage_deposit before this call.
    #[payable]
    pub fn register_new_dao(&mut self, dao_id: AccountId, vote_token_id: AccountId) -> Promise {
        let storage_deposit = env::attached_deposit();
        assert!(storage_deposit >= FT_STORAGE_DEPOSIT, "not enough deposit");
        let storage_before = env::storage_usage();
        let mut account_stats = self.get_account_stats(&dao_id);

        self.last_dao_key_suffix += 1;
        let key = into_storage_key_wrapper_u16(DAO_KEY_PREFIX, self.last_dao_key_suffix);
        let users = UnorderedMap::new(key);
        let total_amount = 0;

        let dao_struct = Dao {
            account_id: dao_id.to_owned(),
            vote_token_id: vote_token_id.clone(),
            users,
            total_amount,
        };
        require!(
            self.daos.insert(&dao_id, &dao_struct).is_none(),
            "dao is already registered"
        );
        let storage_after = env::storage_usage();
        let storage_diff = storage_after - storage_before;
        account_stats.add_storage_used(storage_diff);
        self.save_account_stats(&dao_id, &account_stats);
        Promise::new(vote_token_id)
            .function_call(
                "storage_deposit".to_string(),
                b"{\"registration_only\":true}".to_vec(),
                storage_deposit,
                Gas(10 * 10u64.pow(12)),
            )
            .then(
                ext_self::ext(env::current_account_id())
                    .with_static_gas(Gas(10 * 10u64.pow(12)))
                    .return_deposit(env::predecessor_account_id(), storage_deposit),
            )
    }

    /// Registers caller in dao
    /// Adds DAO's storage used and increases user count by 1.
    /// Requires some deposit to protect dao from malicious users.
    /// This deposit is returned on unregister.
    #[payable]
    pub fn register_in_dao(&mut self, dao_id: AccountId) -> Promise {
        require!(
            env::attached_deposit() >= MIN_REGISTER_DEPOSIT,
            "not enough deposit"
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
        ext_dao::ext(dao_id)
            .with_attached_deposit(
                sender_id.as_bytes().len() as u128
                    * MIN_STORAGE_FOR_DAO as Balance
                    * env::storage_byte_cost(),
            )
            .with_static_gas(GAS_FOR_REGISTER)
            .register_delegation(sender_id.clone())
    }

    /// Unregisters caller from dao.
    /// Caller is supposed to have all his tokens withdrawn.
    /// Frees DAO's storage used and decreases user count by 1.
    /// Returns promise with register deposit transfer to the caller.
    #[payable]
    pub fn unregister_in_dao(&mut self, dao_id: AccountId) -> Promise {
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
        Promise::new(sender_id).transfer(MIN_REGISTER_DEPOSIT)
    }

    /// Delegates `amount` owned tokens to the `delegate_id`.
    pub fn delegate_owned(
        &mut self,
        dao_id: AccountId,
        delegate_id: AccountId,
        amount: U128,
    ) -> Promise {
        let sender_id = env::predecessor_account_id();
        self.internal_delegate_owned(dao_id, sender_id, delegate_id, amount.0)
    }

    /// Undelegates `amount` tokens from `delegate_id`.
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
        ext_dao::ext(dao.account_id)
            .with_static_gas(GAS_FOR_UNDELEGATE)
            .undelegate(delegate_id, amount)
    }

    /// Delegate all delegated tokens from caller's delegators to `delegate_id`.
    /// Once delegated, cannot be undelegated back.
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
        ext_dao::ext(dao.account_id.clone())
            .with_static_gas(GAS_FOR_UNDELEGATE)
            .transfer_amount(sender_id, delegate_id, amount.into())
    }
    /// Withdraw vote tokens.
    /// Only vote amount which is not delegated can be withdrawn.
    pub fn withdraw(&mut self, dao_id: AccountId, amount: U128) -> Promise {
        let sender_id = env::predecessor_account_id();
        let mut dao = self.get_dao(&dao_id);
        dao.user_withdraw(&sender_id, amount.0);
        self.save_dao(&dao_id, &dao);
        ext_fungible_token::ext(dao.vote_token_id.clone())
            .with_static_gas(GAS_FOR_FT_TRANSFER)
            .with_attached_deposit(1)
            .ft_transfer(sender_id.clone(), amount, None)
            .then(
                ext_self::ext(env::current_account_id())
                    .with_static_gas(GAS_FOR_FT_TRANSFER)
                    .exchange_callback_post_withdraw(dao_id, sender_id, amount),
            )
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
            "internal withdraw callback",
        );
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {}
            PromiseResult::Failed => {
                let mut dao = self.get_dao(&dao_id);
                // This reverts the changes from withdraw function.
                dao.user_deposit(&sender_id, amount.0);
                self.save_dao(&dao_id, &dao);
            }
        };
    }
    #[private]
    pub fn return_deposit(&mut self, account_id: AccountId, amount: u128) {
        require!(
            env::promise_results_count() == 1,
            "internal return_deposit callback",
        );
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {}
            PromiseResult::Failed => {
                Promise::new(account_id).transfer(amount);
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
    pub fn dao_user_list(&self, dao_id: AccountId) -> Vec<(AccountId, User)> {
        let dao = self.get_dao(&dao_id);
        dao.users
            .to_vec()
            .into_iter()
            .map(|(account_id, versioned_user)| (account_id, versioned_user.into()))
            .collect()
    }
}

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    /// Method called by FT contract which adds `amount` of vote tokens
    /// to `sender_id` account in dao specified in `msg` as deserialized `TransferMsgInfo` object.
    /// If msg has `delegate_id` key, then all deposited tokens are transfered to it.
    /// Fails if:
    /// - malformed/missing `TransferMsgInfo` object in `msg`
    /// - dao is not registered
    /// - dao does not have caller's account registered as vote token
    /// - sender_id or delegate_id is not registered in dao
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let dao_transfer: TransferMsgInfo =
            serde_json::from_str(msg.as_str()).expect("invalid msg format");
        let mut dao = self.get_dao(&dao_transfer.dao_id);
        require!(
            dao.vote_token_id == env::predecessor_account_id(),
            "invalid token"
        );
        dao.user_deposit(&sender_id, amount.0);
        self.save_dao(&dao_transfer.dao_id, &dao);
        if let Some(delegate_id) = dao_transfer.delegate_id {
            self.internal_delegate_owned(dao_transfer.dao_id, sender_id, delegate_id, amount.0);
        }
        PromiseOrValue::Value(U128(0))
    }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
struct TransferMsgInfo {
    pub dao_id: AccountId,
    pub delegate_id: Option<AccountId>,
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
            .expect("internal storage sub");
    }
    pub fn add_balance(&mut self, amount: Balance) {
        self.near_amount += amount;
    }
    pub fn remove_balance(&mut self, amount: Balance) {
        self.near_amount = self
            .near_amount
            .checked_sub(amount)
            .expect("internal balance sub");
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
            "dao does not have enough storage deposit"
        )
    }
}

#[ext_contract(ext_dao)]
pub trait ExtDao {
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
pub trait ExtSelf {
    fn exchange_callback_post_withdraw(
        &mut self,
        dao_id: AccountId,
        sender_id: AccountId,
        amount: U128,
    );
    fn return_deposit(&self, account_id: AccountId, amount: u128);
}

#[ext_contract(ext_fungible_token)]
pub trait ExtFungibleToken {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
    fn return_deposit(&self, account_id: AccountId, amount: u128);
}

impl Contract {
    pub fn get_dao(&self, dao_id: &AccountId) -> Dao {
        self.daos.get(dao_id).expect("dao not found")
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
            .expect("account not registered")
    }
    pub fn save_account_stats(&mut self, account_id: &AccountId, stats: &AccountStats) {
        self.dao_storage_balance.insert(account_id, stats);
    }
    pub fn internal_delegate_owned(
        &mut self,
        dao_id: AccountId,
        sender_id: AccountId,
        delegate_id: AccountId,
        amount: u128,
    ) -> Promise {
        let storage_before = env::storage_usage();
        let mut dao = self.get_dao(&dao_id);
        dao.delegate_owned(sender_id.clone(), delegate_id.clone(), amount);
        let mut account_stats = self.get_account_stats(&dao_id);
        self.save_dao(&dao_id, &dao);
        let storage_after = env::storage_usage();
        account_stats.add_storage_used(storage_after - storage_before);
        self.save_account_stats(&dao_id, &account_stats);
        account_stats.assert_enough_deposit();
        ext_dao::ext(dao.account_id)
            .with_static_gas(GAS_FOR_DELEGATE)
            .delegate_owned(delegate_id, amount.into())
    }
}
