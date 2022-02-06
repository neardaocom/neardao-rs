use std::u128;

use crate::constants::MAX_FT_TOTAL_SUPPLY;
use crate::settings::{assert_valid_dao_settings, DaoSettings, VDaoSettings};
use crate::standard_impl::ft::FungibleToken;
use crate::standard_impl::ft_metadata::{FungibleTokenMetadata, FungibleTokenMetadataProvider};
use crate::tags::{TagInput, Tags};
use library::storage::StorageBucket;
use library::workflow::{Instance, ProposeSettings, Template, TemplateSettings};
use near_contract_standards::fungible_token::core::FungibleTokenCore;
use near_contract_standards::fungible_token::resolver::FungibleTokenResolver;
use near_contract_standards::storage_management::{
    StorageBalance, StorageBalanceBounds, StorageManagement,
};

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap};
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::{
    env, near_bindgen, AccountId, BorshStorageKey, IntoStorageKey, PanicOnDefault, Promise,
    PromiseOrValue,
};

use crate::group::{Group, GroupInput};

use crate::media::Media;
use crate::FnCallId;
use crate::{action::*, GroupId};
use crate::{proposal::*, StorageKey, TagCategory};

near_sdk::setup_alloc!();

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    FT,
    FTMetadata,
    Proposals,
    ProposalConfig,
    Council,
    Tags,
    Media,
    FunctionCalls,
    FunctionCallMetadata,
    Storage,
    DaoSettings,
    VoteSettings,
    VConfig,
    ReleaseConfig,
    ReleaseDb,
    DocMetadata,
    Mappers,
    NewVersionCode,
    FactoryAcc,
    TokenGroupRights,
    UserRights,
    StorageDeposit,
    RefPools,
    SkywardAuctions,
    Groups,
    Rights,
    FunctionCallWhitelist,
    WfTemplate,
    WfTemplateSettings,
    ProposedWfTemplateSettings, //for proposal workflow add template
    WfInstance,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub ft_total_supply: u32,
    pub ft_total_locked: u32,
    pub ft_total_distributed: u32,
    pub total_members_count: u32,
    pub decimal_const: u128,
    pub ft: FungibleToken,
    pub ft_metadata: LazyOption<FungibleTokenMetadata>,
    pub group_last_id: GroupId,
    pub groups: UnorderedMap<GroupId, Group>,
    pub settings: LazyOption<VDaoSettings>,
    pub proposal_last_id: u32,
    pub proposals: UnorderedMap<u32, VProposal>,
    pub storage: UnorderedMap<StorageKey, StorageBucket>,
    pub tags: UnorderedMap<TagCategory, Tags>,
    pub media_last_id: u32,
    pub media: LookupMap<u32, Media>, //TODO categorize??
    pub function_call_metadata: LookupMap<FnCallId, Vec<FnCallMetadata>>,
    pub function_calls: UnorderedMap<FnCallId, FnCallDefinition>,
    pub workflow_last_id: u16,
    pub workflow_template: UnorderedMap<u16, (Template, Vec<TemplateSettings>)>,
    pub workflow_instance: UnorderedMap<u32, (Instance, ProposeSettings)>,
    pub proposed_workflow_settings: LookupMap<u32, Vec<TemplateSettings>>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        total_supply: u32,
        ft_metadata: FungibleTokenMetadata,
        settings: DaoSettings,
        groups: Vec<GroupInput>,
        media: Vec<Media>,
        tags: Vec<TagInput>,
        function_calls: Vec<FnCallDefinition>,
        function_call_metadata: Vec<Vec<FnCallMetadata>>,
        workflow_templates: Vec<Template>,
        workflow_template_settings: Vec<Vec<TemplateSettings>>,
    ) -> Self {
        assert!(total_supply <= MAX_FT_TOTAL_SUPPLY);
        assert_valid_dao_settings(&settings);

        let mut contract = Contract {
            ft_total_supply: total_supply,
            ft_total_locked: 0,
            ft_total_distributed: 0,
            total_members_count: 0,
            decimal_const: 10u128.pow(ft_metadata.decimals as u32),
            ft: FungibleToken::new(StorageKeys::FT),
            ft_metadata: LazyOption::new(StorageKeys::FTMetadata, Some(&ft_metadata)),
            settings: LazyOption::new(StorageKeys::DaoSettings, None),
            group_last_id: 0,
            groups: UnorderedMap::new(StorageKeys::Groups),
            proposal_last_id: 0,
            proposals: UnorderedMap::new(StorageKeys::Proposals),
            storage: UnorderedMap::new(StorageKeys::Storage),
            tags: UnorderedMap::new(StorageKeys::Tags),
            media_last_id: 0,
            media: LookupMap::new(StorageKeys::Media),
            function_call_metadata: LookupMap::new(StorageKeys::FunctionCallMetadata),
            function_calls: UnorderedMap::new(StorageKeys::FunctionCalls),
            workflow_last_id: 0,
            workflow_template: UnorderedMap::new(StorageKeys::WfTemplate),
            workflow_instance: UnorderedMap::new(StorageKeys::WfInstance),
            proposed_workflow_settings: LookupMap::new(StorageKeys::ProposedWfTemplateSettings),
        };

        //register self and mint all FT
        let contract_acc = env::current_account_id();
        contract.ft.internal_register_account(&contract_acc);
        contract.ft.internal_deposit(
            &contract_acc,
            contract.ft_total_supply as u128 * contract.decimal_const,
        );

        contract.init_dao_settings(settings);
        contract.init_tags(tags);
        contract.init_groups(groups);
        contract.init_media(media);
        contract.init_function_calls(function_calls, function_call_metadata);
        contract.init_workflows(workflow_templates, workflow_template_settings);

        contract
    }

    /// For dev/testing purposes only
    #[cfg(feature = "testnet")]
    pub fn clean_self(&mut self) {
        env::storage_remove(&StorageKeys::NewVersionCode.into_storage_key());
    }

    /// For dev/testing purposes only
    #[cfg(feature = "testnet")]
    pub fn delete_self(self) -> Promise {
        let settings: DaoSettings = self.settings.get().unwrap().into();
        Promise::new(env::current_account_id()).delete_account(settings.dao_admin_account_id)
    }
}

/******************************************************************************
 *
 * Fungible Token (NEP-141)
 * https://nomicon.io/Standards/FungibleToken/Core.html
 *
 ******************************************************************************/

#[near_bindgen]
impl FungibleTokenCore for Contract {
    #[payable]
    fn ft_transfer(&mut self, receiver_id: ValidAccountId, amount: U128, memo: Option<String>) {
        self.ft.ft_transfer(receiver_id, amount, memo)
    }

    #[payable]
    fn ft_transfer_call(
        &mut self,
        receiver_id: ValidAccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128> {
        self.ft.ft_transfer_call(receiver_id, amount, memo, msg)
    }

    fn ft_total_supply(&self) -> U128 {
        self.ft.ft_total_supply()
    }

    fn ft_balance_of(&self, account_id: ValidAccountId) -> U128 {
        self.ft.ft_balance_of(account_id)
    }
}

#[near_bindgen]
impl FungibleTokenResolver for Contract {
    #[private]
    fn ft_resolve_transfer(
        &mut self,
        sender_id: ValidAccountId,
        receiver_id: ValidAccountId,
        amount: U128,
    ) -> U128 {
        let sender_id: AccountId = sender_id.into();
        let (used_amount, burned_amount) =
            self.ft
                .internal_ft_resolve_transfer(&sender_id, receiver_id, amount);
        if burned_amount > 0 {
            //self.on_tokens_burned(sender_id, burned_amount);
        }
        used_amount.into()
    }
}

/******************************************************************************
 *
 * Fungible Token Metadata (NEP-148)
 * https://nomicon.io/Standards/FungibleToken/Metadata.html
 *
 ******************************************************************************/

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.ft_metadata.get().unwrap()
    }
}

/******************************************************************************
 *
 * Storage Management (NEP-145)
 * https://nomicon.io/Standards/StorageManagement.html
 *
 ******************************************************************************/

#[near_bindgen]
impl StorageManagement for Contract {
    #[payable]
    fn storage_deposit(
        &mut self,
        account_id: Option<ValidAccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance {
        self.ft.storage_deposit(account_id, registration_only)
    }

    #[payable]
    fn storage_withdraw(&mut self, amount: Option<U128>) -> StorageBalance {
        self.ft.storage_withdraw(amount)
    }

    #[payable]
    fn storage_unregister(&mut self, force: Option<bool>) -> bool {
        #[allow(unused_variables)]
        if let Some((account_id, balance)) = self.ft.internal_storage_unregister(force) {
            //self.on_account_closed(account_id, balance);
            true
        } else {
            false
        }
    }

    fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        self.ft.storage_balance_bounds()
    }

    fn storage_balance_of(&self, account_id: ValidAccountId) -> Option<StorageBalance> {
        self.ft.storage_balance_of(account_id)
    }
}

/// Triggers new version download from factory
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn download_new_version() {
    use crate::constants::{GAS_MIN_DOWNLOAD_LIMIT, VERSION};
    use env::BLOCKCHAIN_INTERFACE;

    env::setup_panic_hook();
    env::set_blockchain_interface(Box::new(near_blockchain::NearBlockchain {}));

    // We are not able to access council members any other way so we have deserialize SC
    let contract: Contract = env::state_read().unwrap();
    let dao_settings: DaoSettings = contract.settings.get().unwrap().into();

    //TODO who can trigger the download
    assert_eq!(
        dao_settings.dao_admin_account_id,
        env::predecessor_account_id()
    );

    let factory_acc = env::storage_read(&StorageKeys::FactoryAcc.into_storage_key()).unwrap();
    let method_name = b"download_dao_bin".to_vec();

    unsafe {
        BLOCKCHAIN_INTERFACE.with(|b| {
            b.borrow().as_ref().unwrap().promise_create(
                factory_acc.len() as _,
                factory_acc.as_ptr() as _,
                method_name.len() as _,
                method_name.as_ptr() as _,
                1 as _,
                [VERSION].to_vec().as_ptr() as _,
                0,
                GAS_MIN_DOWNLOAD_LIMIT,
            );
        });
    }
}

/// Method called by dao factory as response to download_new_version method
/// Saves provided dao binary in storage under "NewVersionCode"
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn store_new_version() {
    env::setup_panic_hook();
    env::set_blockchain_interface(Box::new(near_blockchain::NearBlockchain {}));
    assert_eq!(
        env::predecessor_account_id(),
        String::from_utf8(env::storage_read(&StorageKeys::FactoryAcc.into_storage_key()).unwrap())
            .unwrap()
            .to_string()
    );
    env::storage_write(
        &StorageKeys::NewVersionCode.into_storage_key(),
        &env::input().unwrap(),
    );
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn upgrade_self() {
    use crate::constants::GAS_MIN_UPGRADE_LIMIT;
    use near_sdk::env::BLOCKCHAIN_INTERFACE;

    env::setup_panic_hook();
    env::set_blockchain_interface(Box::new(near_blockchain::NearBlockchain {}));

    // We are not able to access council members any other way so we have deserialize SC
    let contract: Contract = env::state_read().unwrap();
    let dao_settings: DaoSettings = contract.settings.get().unwrap().into();

    assert_eq!(
        dao_settings.dao_admin_account_id,
        env::predecessor_account_id()
    );

    let current_acc = env::current_account_id().into_bytes();
    let method_name = "migrate".as_bytes().to_vec();
    let key = StorageKeys::NewVersionCode.into_storage_key();

    unsafe {
        BLOCKCHAIN_INTERFACE.with(|b| {
            // Load stored wasm code into register 0.
            b.borrow()
                .as_ref()
                .unwrap()
                .storage_read(key.len() as _, key.as_ptr() as _, 0);
            // schedule a Promise tx to this same contract
            let promise_id = b
                .borrow()
                .as_ref()
                .unwrap()
                .promise_batch_create(current_acc.len() as _, current_acc.as_ptr() as _);
            // 1st item in the Tx: "deploy contract" (code is taken from register 0)
            b.borrow()
                .as_ref()
                .unwrap()
                .promise_batch_action_deploy_contract(promise_id, u64::MAX as _, 0);
            // 2nd item in the Tx: call this account's migrate() method
            b.borrow()
                .as_ref()
                .unwrap()
                .promise_batch_action_function_call(
                    promise_id,
                    method_name.len() as _,
                    method_name.as_ptr() as _,
                    0 as _,
                    0 as _,
                    0 as _,
                    GAS_MIN_UPGRADE_LIMIT,
                );
        });
    }
}

pub struct StorageKeyWrapper(pub Vec<u8>);

impl IntoStorageKey for StorageKeyWrapper {
    fn into_storage_key(self) -> Vec<u8> {
        self.0
    }
}

impl From<Vec<u8>> for StorageKeyWrapper {
    fn from(bytes: Vec<u8>) -> StorageKeyWrapper {
        StorageKeyWrapper(bytes)
    }
}
