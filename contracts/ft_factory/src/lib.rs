use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::UnorderedSet,
    env, ext_contract,
    json_types::{Base64VecU8, U128},
    log, near_bindgen, AccountId, BorshStorageKey, Gas, PanicOnDefault, Promise,
};

/// Gas spent on the call & account creation.
const CREATE_CALL_GAS: Gas = Gas(75_000_000_000_000);
/// Gas allocated on the callback.
const ON_CREATE_CALL_GAS: Gas = Gas(10_000_000_000_000);

const FUNGIBLE_TOKEN_WASM: &[u8] = include_bytes!("../../../res/fungible_token.wasm");

const DEPOSIT_CREATE: u128 = 2_200_000_000_000_000_000_000_000;

#[ext_contract(ext_self)]
pub trait ExtSelf {
    fn on_create(
        &mut self,
        account_id: AccountId,
        attached_deposit: U128,
        predecessor_account_id: AccountId,
    ) -> bool;
}

#[derive(BorshStorageKey, BorshSerialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
pub enum StorageKeys {
    DeployedTokens,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub accounts: UnorderedSet<AccountId>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        Self {
            accounts: UnorderedSet::new(StorageKeys::DeployedTokens),
        }
    }
    #[payable]
    pub fn create(&mut self, name: String, args: Base64VecU8) -> Promise {
        assert!(
            env::attached_deposit() >= DEPOSIT_CREATE,
            "Not enough attached deposit."
        );
        let account_id: AccountId = format!("{}.{}", name, env::current_account_id())
            .try_into()
            .expect("Account is not valid.");
        assert!(
            !self.accounts.contains(&account_id),
            "{}",
            "Account already exists."
        );
        let promise = Promise::new(account_id.clone())
            .create_account()
            .deploy_contract(FUNGIBLE_TOKEN_WASM.to_vec())
            .transfer(env::attached_deposit());
        promise
            .function_call(
                "new".to_string(),
                args.into(),
                0,
                env::prepaid_gas() - CREATE_CALL_GAS - ON_CREATE_CALL_GAS,
            )
            .then(ext_self::on_create(
                account_id,
                U128(env::attached_deposit()),
                env::predecessor_account_id(),
                env::current_account_id(),
                0,
                ON_CREATE_CALL_GAS,
            ))
    }
    pub fn on_create(
        &mut self,
        account_id: AccountId,
        attached_deposit: U128,
        predecessor_account_id: AccountId,
    ) -> bool {
        if near_sdk::is_promise_success() {
            self.accounts.insert(&account_id);
            log!("Created fungible token contract: {}", &account_id);
            true
        } else {
            Promise::new(predecessor_account_id).transfer(attached_deposit.0);
            false
        }
    }
}
