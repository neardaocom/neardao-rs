/*!
Fungible Token implementation with JSON serialization.
NOTES:
  - The maximum balance value is limited by U128 (2**128 - 1).
  - JSON calls should pass U128 as a base-10 string. E.g. "100".
  - The contract optimizes the inner trie structure by hashing account IDs. It will prevent some
    abuse of deep tries. Shouldn't be an issue, once NEAR clients implement full hashing of keys.
  - The contract tracks the change in storage before and after the call. If the storage increases,
    the contract requires the caller of the contract to attach enough deposit to the function call
    to cover the storage cost.
    This is done to prevent a denial of service attack on the contract by taking all available storage.
    If the storage decreases, the contract will issue a refund for the cost of the released storage.
    The unused tokens from the attached deposit are also refunded, so it's safe to
    attach more deposit than required.
  - To prevent the deployed contract from being modified or deleted, it should not have any access
    keys on its account.
*/
use near_contract_standards::fungible_token::core::FungibleTokenCore;
use near_contract_standards::fungible_token::events::FtMint;
use near_contract_standards::fungible_token::resolver::FungibleTokenResolver;
use near_contract_standards::storage_management::{
    StorageBalance, StorageBalanceBounds, StorageManagement,
};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, log, near_bindgen, require, AccountId, Balance, BorshStorageKey, Gas, PanicOnDefault,
    PromiseOrValue,
};

use standard_impl::impl_ft_metadata::{FungibleTokenMetadata, FungibleTokenMetadataProvider};
use standard_impl::impl_fungible_token::FungibleToken;

mod standard_impl;

pub const VERSION: u8 = 1;
pub const GAS_DOWNLOAD_NEW_VERSION: Gas = Gas(200_000_000_000_000);
pub const GAS_UPGRADE: Gas = Gas(200_000_000_000_000);

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    NewVersionCode,
    Token,
    TokenMeta,
    Settings,
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct InitDistribution {
    pub account_id: AccountId,
    pub amount: U128,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    token: FungibleToken,
    metadata: LazyOption<FungibleTokenMetadata>,
    settings: LazyOption<Settings>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Settings {
    /// Account allowed to change these settings.
    owner_id: AccountId,
    mint_allowed: bool,
    burn_allowed: bool,
    /// Account of contract allowed to provide new version.
    /// If not set then upgrade is not allowed.
    /// TODO: Implement.
    upgrade_provider: Option<AccountId>,
}

#[near_bindgen]
impl Contract {
    /// Initializes the contract with the given total supply owned by the given `owner_id`
    /// with the given fungible token metadata and settings.
    /// Requires `total_supply` makes at least one integer token given metadata decimals.
    /// Distributes amount to account_id in provided via `init_distribution`.
    #[init]
    pub fn new(
        owner_id: AccountId,
        total_supply: U128,
        metadata: FungibleTokenMetadata,
        settings: Option<Settings>,
        init_distribution: Vec<InitDistribution>,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized.");
        assert!(
            total_supply.0 / 10u128.pow(metadata.decimals as u32) >= 1,
            "Invalid total_supply/decimals ratio."
        );
        metadata.assert_valid();
        let settings = settings.unwrap_or_else(|| Settings {
            owner_id: owner_id.clone(),
            mint_allowed: false,
            burn_allowed: false,
            upgrade_provider: None,
        });
        let mut this = Self {
            token: FungibleToken::new(StorageKeys::Token),
            metadata: LazyOption::new(StorageKeys::TokenMeta, Some(&metadata)),
            settings: LazyOption::new(StorageKeys::Settings, Some(&settings)),
        };
        this.token.internal_register_account(&owner_id);
        this.token.internal_deposit(&owner_id, total_supply.into());
        if !init_distribution.is_empty() {
            let memo = "init distribution".to_string();
            for d in init_distribution {
                this.token.internal_register_account(&d.account_id);
                this.token.internal_transfer(
                    &owner_id,
                    &d.account_id,
                    d.amount.0,
                    Some(memo.clone()),
                );
            }
        }
        FtMint {
            owner_id: &owner_id,
            amount: &total_supply,
            memo: Some("Initial tokens supply is minted."),
        }
        .emit();
        this
    }

    fn on_account_closed(&mut self, account_id: AccountId, balance: Balance) {
        log!("Closed @{} with {}", account_id, balance);
    }

    fn on_tokens_burned(&mut self, account_id: AccountId, amount: Balance) {
        log!("Account @{} burned {}", account_id, amount);
    }

    /// Changes current settings to `settings` provided.
    /// Only owner is allowed to call this function.
    pub fn change_settings(&mut self, settings: Settings) {
        let prev_settings = self.settings.get().expect("No settings.");
        require!(
            prev_settings.owner_id == env::predecessor_account_id(),
            "No rights."
        );
        self.settings.set(&settings);
    }

    pub fn mint_new_ft(&mut self, amount: Balance, msg: Option<String>) {
        let settings = self.settings.get().expect("No settings.");
        require!(
            settings.owner_id == env::predecessor_account_id(),
            "No rights."
        );
        require!(settings.mint_allowed, "Minting new tokens is not allowed.");
        self.token.internal_deposit(&settings.owner_id, amount);
        let msg = format!("Minted {} new tokens. {}", amount, msg.unwrap_or_default());
        FtMint {
            owner_id: &settings.owner_id,
            amount: &U128(amount),
            memo: Some(msg.as_str()),
        }
        .emit();
    }

    pub fn settings(&self) -> Option<Settings> {
        self.settings.get()
    }

    /// Returns balances of provided accounts in same order.
    pub fn ft_balances_of(&self, account_ids: Vec<AccountId>) -> Vec<(AccountId, U128)> {
        let mut result = Vec::with_capacity(account_ids.len());
        for acc in account_ids {
            let amount = self.token.accounts.get(&acc).unwrap_or_default();
            result.push((acc, amount.into()));
        }
        result
    }
}

#[near_bindgen]
impl FungibleTokenCore for Contract {
    #[payable]
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>) {
        self.token.ft_transfer(receiver_id, amount, memo)
    }

    #[payable]
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128> {
        self.token.ft_transfer_call(receiver_id, amount, memo, msg)
    }

    fn ft_total_supply(&self) -> U128 {
        self.token.ft_total_supply()
    }

    fn ft_balance_of(&self, account_id: AccountId) -> U128 {
        self.token.ft_balance_of(account_id)
    }
}

#[near_bindgen]
impl FungibleTokenResolver for Contract {
    #[private]
    fn ft_resolve_transfer(
        &mut self,
        sender_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) -> U128 {
        let (used_amount, burned_amount) =
            self.token
                .internal_ft_resolve_transfer(&sender_id, receiver_id, amount);
        if burned_amount > 0 {
            self.on_tokens_burned(sender_id, burned_amount);
        }
        used_amount.into()
    }
}

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}

#[near_bindgen]
impl StorageManagement for Contract {
    #[payable]
    fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance {
        self.token.storage_deposit(account_id, registration_only)
    }

    #[payable]
    fn storage_withdraw(&mut self, amount: Option<U128>) -> StorageBalance {
        self.token.storage_withdraw(amount)
    }

    #[payable]
    fn storage_unregister(&mut self, force: Option<bool>) -> bool {
        #[allow(unused_variables)]
        if let Some((account_id, balance)) = self.token.internal_storage_unregister(force) {
            self.on_account_closed(account_id, balance);
            true
        } else {
            false
        }
    }

    fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        self.token.storage_balance_bounds()
    }

    fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBalance> {
        self.token.storage_balance_of(account_id)
    }
}

/// Triggers new version download from upgrade provider.
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn download_new_version() {
    env::setup_panic_hook();

    // We are not able to access council members any other way so we have deserialize SC
    let contract: Contract = env::state_read().unwrap();
    let settings: Settings = contract.settings().unwrap();

    require!(
        settings.owner_id == env::predecessor_account_id(),
        "no rights"
    );

    let factory_acc = settings.upgrade_provider.expect("upgrade is not allowed");
    let method_name = "download_ft_bin";

    env::promise_create(
        factory_acc,
        method_name,
        &[VERSION],
        0,
        GAS_DOWNLOAD_NEW_VERSION,
    );
}

/// Method called by upgrade provider as response to download_new_version method.
/// Saves provided binary blob in storage under `StorageKey::NewVersionCode`.
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn store_new_version() {
    use near_sdk::IntoStorageKey;

    env::setup_panic_hook();

    let contract: Contract = env::state_read().unwrap();
    let settings: Settings = contract.settings().unwrap();
    require!(
        settings.upgrade_provider.expect("upgrade is not allowed") == env::predecessor_account_id(),
        "no rights"
    );
    env::storage_write(
        &StorageKeys::NewVersionCode.into_storage_key(),
        &env::input().unwrap(),
    );
}

// TODO: Use near-sys to access low-level interface.
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn upgrade_self() {
    use near_sdk::IntoStorageKey;

    env::setup_panic_hook();

    // We are not able to access council members any other way so we have deserialize SC
    let contract: Contract = env::state_read().unwrap();
    let settings: Settings = contract.settings().unwrap();

    require!(
        settings.owner_id == env::predecessor_account_id(),
        "no rights"
    );

    let current_acc = env::current_account_id();
    let method_name = "upgrade";
    let key = StorageKeys::NewVersionCode.into_storage_key();

    let code = env::storage_read(key.as_slice()).expect("Failed to read new code from storage.");
    let promise = env::promise_batch_create(&current_acc);
    env::promise_batch_action_deploy_contract(promise, code.as_slice());
    env::promise_batch_action_function_call(promise, method_name, &[], 0, GAS_UPGRADE);
}

// TODO: Add more tests.
#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    #[allow(unused)]
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, Balance};

    use super::*;

    const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 288 288'%3E%3Cg id='l' data-name='l'%3E%3Cpath d='M187.58,79.81l-30.1,44.69a3.2,3.2,0,0,0,4.75,4.2L191.86,103a1.2,1.2,0,0,1,2,.91v80.46a1.2,1.2,0,0,1-2.12.77L102.18,77.93A15.35,15.35,0,0,0,90.47,72.5H87.34A15.34,15.34,0,0,0,72,87.84V201.16A15.34,15.34,0,0,0,87.34,216.5h0a15.35,15.35,0,0,0,13.08-7.31l30.1-44.69a3.2,3.2,0,0,0-4.75-4.2L96.14,186a1.2,1.2,0,0,1-2-.91V104.61a1.2,1.2,0,0,1,2.12-.77l89.55,107.23a15.35,15.35,0,0,0,11.71,5.43h3.13A15.34,15.34,0,0,0,216,201.16V87.84A15.34,15.34,0,0,0,200.66,72.5h0A15.35,15.35,0,0,0,187.58,79.81Z'/%3E%3C/g%3E%3C/svg%3E";
    pub const FT_METADATA_SPEC: &str = "ft-1.0.0";

    fn default_ft_metadata() -> FungibleTokenMetadata {
        FungibleTokenMetadata {
            spec: FT_METADATA_SPEC.to_string(),
            name: "Example NEAR fungible token".to_string(),
            symbol: "EXAMPLE".to_string(),
            icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
            reference: None,
            reference_hash: None,
            decimals: 24,
        }
    }

    const TOTAL_SUPPLY: Balance = 1_000_000_000 * 10u128.pow(24);

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn test_new() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let contract = Contract::new(
            accounts(1).into(),
            TOTAL_SUPPLY.into(),
            default_ft_metadata(),
            None,
            vec![
                InitDistribution {
                    account_id: accounts(2),
                    amount: U128(1_000),
                },
                InitDistribution {
                    account_id: accounts(3),
                    amount: U128(7_000),
                },
                InitDistribution {
                    account_id: accounts(4),
                    amount: U128(2_000),
                },
            ],
        );
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.ft_total_supply().0, TOTAL_SUPPLY);
        assert_eq!(contract.ft_balance_of(accounts(1)).0, TOTAL_SUPPLY - 10_000);
        assert_eq!(contract.ft_balance_of(accounts(2)).0, 1_000);
        assert_eq!(contract.ft_balance_of(accounts(3)).0, 7_000);
        assert_eq!(contract.ft_balance_of(accounts(4)).0, 2_000);
    }

    #[test]
    #[should_panic(expected = "The contract is not initialized")]
    fn test_default() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let _contract = Contract::default();
    }

    #[test]
    fn test_transfer() {
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract = Contract::new(
            accounts(2).into(),
            TOTAL_SUPPLY.into(),
            default_ft_metadata(),
            None,
            vec![],
        );
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.storage_balance_bounds().min.into())
            .predecessor_account_id(accounts(1))
            .build());
        // Paying for account registration, aka storage deposit
        contract.storage_deposit(None, None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(2))
            .build());
        let transfer_amount = TOTAL_SUPPLY / 3;
        contract.ft_transfer(accounts(1), transfer_amount.into(), None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert_eq!(
            contract.ft_balance_of(accounts(2)).0,
            (TOTAL_SUPPLY - transfer_amount)
        );
        assert_eq!(contract.ft_balance_of(accounts(1)).0, transfer_amount);
    }
}
