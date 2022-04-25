use consts::{
    ACCOUNT_MAX_LENGTH, DAO_KEY_PREFIX, GAS_FOR_DELEGATE, GAS_FOR_FT_TRANSFER, GAS_FOR_REGISTER,
    GAS_FOR_UNDELEGATE, U128_LEN, U64_LEN,
};
use dao::Dao;
use library::functions::utils::into_storage_key_wrapper_u16;
use near_contract_standards::fungible_token::core_impl::ext_fungible_token;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, ext_contract, near_bindgen, serde_json, AccountId, Balance, BorshStorageKey,
    PanicOnDefault, Promise, PromiseOrValue, PromiseResult, StorageUsage,
};

pub use user::{User, VersionedUser};

mod consts;
mod dao;
// mod storage_impl;
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
    storage_deposit: LookupMap<AccountId, (Balance, StorageUsage)>,
    /// Suffix used for new DAOs to avoid storage keys collision.
    last_dao_key_suffix: u16,
}

#[near_bindgen]
impl Contract {
    // TODO: Measure real value with a test.
    /// Minimum storage with empty delegations in bytes.
    /// This includes u128 stored in DAO for delegations to this user.
    /// They are deposited on internal_register and removed on internal_unregister.
    pub fn min_storage_per_dao() -> StorageUsage {
        2 * ACCOUNT_MAX_LENGTH + U64_LEN + 4 * U128_LEN
    }

    #[init]
    pub fn new(registrar_id: AccountId) -> Self {
        Self {
            registrar_id,
            daos: LookupMap::new(StorageKeys::Daos),
            storage_deposit: LookupMap::new(StorageKeys::StorageDeposit),
            last_dao_key_suffix: 0,
        }
    }

    /// Registers new dao in contract.
    pub fn register_new_dao(&mut self, dao_id: AccountId, vote_token_id: AccountId) {
        assert_eq!(
            env::predecessor_account_id(),
            self.registrar_id,
            "Only owner can call this method"
        );

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

        self.daos
            .insert(&dao_id, &dao_struct)
            .expect("Dao already exists");
    }

    /// Delegates owned tokens.
    pub fn delegate_owned(
        &mut self,
        dao_id: AccountId,
        delegate_id: AccountId,
        amount: U128,
    ) -> Promise {
        let sender_id = env::predecessor_account_id();
        let mut dao = self.get_dao(&dao_id);
        dao.delegate_owned(sender_id, delegate_id.clone(), amount.0);
        let promise = ext_dao::delegate_owned(
            delegate_id,
            amount,
            dao.account_id.clone(),
            0,
            GAS_FOR_DELEGATE,
        );
        self.save_dao(&dao_id, &dao);
        promise
    }
    /// Undelegates tokens.
    pub fn undelegate(
        &mut self,
        dao_id: AccountId,
        delegate_id: AccountId,
        amount: U128,
    ) -> Promise {
        let sender_id = env::predecessor_account_id();
        let mut dao = self.get_dao(&dao_id);
        dao.undelegate(sender_id, delegate_id.clone(), amount.0);
        let promise = ext_dao::undelegate(
            delegate_id,
            amount,
            dao.account_id.clone(),
            0,
            GAS_FOR_UNDELEGATE,
        );
        self.save_dao(&dao_id, &dao);
        promise
    }
    /// Delegate all delegated tokens to delegate.
    pub fn delegate(&mut self, dao_id: AccountId, delegate_id: AccountId) -> Promise {
        let sender_id = env::predecessor_account_id();
        let mut dao = self.get_dao(&dao_id);
        let amount = dao.delegate(&sender_id, delegate_id.clone());
        self.save_dao(&dao_id, &dao);
        let promise = ext_dao::delegate(
            sender_id,
            delegate_id,
            amount.into(),
            dao.account_id.clone(),
            0,
            GAS_FOR_UNDELEGATE,
        );
        self.save_dao(&dao_id, &dao);
        promise
    }

    /// Withdraw staked tokens.
    pub fn withdraw(&mut self, dao_id: AccountId, amount: U128) -> Promise {
        let sender_id = env::predecessor_account_id();
        let mut dao = self.get_dao(&dao_id);
        dao.user_withdraw(&sender_id, amount.0);
        let promise = ext_fungible_token::ft_transfer(
            sender_id.clone(),
            amount,
            None,
            dao.vote_token_id.clone(),
            1,
            GAS_FOR_FT_TRANSFER,
        )
        .then(ext_self::exchange_callback_post_withdraw(
            sender_id,
            amount,
            env::current_account_id(),
            0,
            GAS_FOR_FT_TRANSFER,
        ));
        self.save_dao(&dao_id, &dao);
        promise
    }

    #[private]
    pub fn exchange_callback_post_withdraw(
        &mut self,
        dao_id: AccountId,
        sender_id: AccountId,
        amount: U128,
    ) {
        assert_eq!(
            env::promise_results_count(),
            1,
            "ERR_CALLBACK_POST_WITHDRAW_INVALID",
        );
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {}
            PromiseResult::Failed => {
                let mut dao = self.get_dao(&dao_id);
                // This reverts the changes from withdraw function.
                dao.user_deposit(sender_id, amount.0);
            }
        };
    }

    /// Total staked amount in dao.
    pub fn dao_ft_total_supply(&self, dao_id: AccountId) -> U128 {
        let dao = self.get_dao(&dao_id);
        dao.ft_total_supply()
    }

    /// Total number of tokens staked by given user in dao.
    pub fn dao_ft_balance_of(&self, dao_id: AccountId, user_acc: AccountId) -> U128 {
        let dao = self.get_dao(&dao_id);
        dao.ft_balance_of(user_acc)
    }

    /// Returns user information.
    pub fn dao_get_user(&self, dao_id: AccountId, user_acc: AccountId) -> User {
        let dao = self.get_dao(&dao_id);
        dao.get_user(&user_acc)
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

        assert_eq!(
            dao.vote_token_id,
            env::predecessor_account_id(),
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

#[ext_contract(ext_dao)]
pub trait Contract {
    fn register_delegation(&mut self, account_id: AccountId);
    fn delegate_owned(&mut self, account_id: AccountId, amount: U128);
    fn undelegate(&mut self, account_id: AccountId, amount: U128);
    fn delegate(&mut self, prev_account_id: AccountId, new_account_id: AccountId, amount: U128);
}

#[ext_contract(ext_self)]
pub trait Contract {
    fn exchange_callback_post_withdraw(&mut self, sender_id: AccountId, amount: U128);
}

impl Contract {
    pub fn get_dao(&self, dao_id: &AccountId) -> Dao {
        self.daos.get(dao_id).expect("Dao not found")
    }
    pub fn save_dao(&mut self, dao_id: &AccountId, dao: &Dao) -> Option<Dao> {
        self.daos.insert(dao_id, &dao)
    }

    pub fn register_user(
        &mut self,
        dao_id: &AccountId,
        owner_id: &AccountId,
        sender_id: &AccountId,
        near_amount: Balance,
    ) {
        let mut dao = self.get_dao(dao_id);
        dao.register_user(owner_id, sender_id, near_amount);
        ext_dao::register_delegation(
            sender_id.clone(),
            owner_id.clone(),
            (U128_LEN as Balance) * env::storage_byte_cost(),
            GAS_FOR_REGISTER,
        );
        self.save_dao(dao_id, &dao);
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
