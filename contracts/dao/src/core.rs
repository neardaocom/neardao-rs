#![allow(clippy::too_many_arguments)]

use crate::constants::GLOBAL_BUCKET_IDENT;
use crate::reward::VersionedReward;
use crate::role::{Roles, UserRoles};
use crate::settings::{assert_valid_dao_settings, Settings, VersionedSettings};
use crate::tags::{TagInput, Tags};
use crate::treasury::{TreasuryPartitionInput, VersionedTreasuryPartition};
use crate::wallet::VersionedWallet;
use library::storage::StorageBucket;
use library::workflow::instance::Instance;
use library::workflow::settings::{ProposeSettings, TemplateSettings};
use library::workflow::template::Template;
use library::workflow::types::{DaoActionIdent, ObjectMetadata};
use library::{FnCallId, MethodName};

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap};
use near_sdk::serde::Serialize;
use near_sdk::{
    env, near_bindgen, AccountId, Balance, BorshStorageKey, IntoStorageKey, PanicOnDefault,
};

use crate::group::{Group, GroupInput};

use crate::{proposal::*, StorageKey, TagCategory};
use crate::{GroupId, ProposalId};

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
// TODO: Remove.
pub struct ActivityLog {
    pub caller: AccountId,
    pub action_id: u8,
    pub timestamp: u64,
}

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    Delegations,
    Events,
    UserRoles,
    GroupRoles,
    Proposals,
    Tags,
    FunctionCalls,
    FunctionCallMetadata,
    StandardFunctionCallMetadata,
    Storage,
    DaoSettings,
    NewVersionCode,
    Groups,
    FunctionCallWhitelist,
    WfTemplate,
    WfTemplateSettings,
    WfProposeSettings,
    ProposedWfTemplateSettings,
    ActivityLog,
    WfInstance,
    DaoActionMetadata,
    TreasuryPartition,
    Wallet,
    Rewards,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    /// Delegations per user.
    pub delegations: LookupMap<AccountId, Balance>,
    /// Total count of unique delegators.
    pub total_delegators_count: u32,
    /// Delegated token total amount.
    pub total_delegation_amount: Balance,
    /// User's roles in groups.
    pub user_roles: LookupMap<AccountId, UserRoles>,
    /// Group's provided roles.
    pub group_roles: LookupMap<GroupId, Roles>,
    /// Total amount of minted tokens.
    pub ft_total_supply: u32,
    /// Decimals of token.
    pub decimals: u8,
    pub ft_total_locked: u32,
    pub ft_total_distributed: u32,
    /// Sum all unique members in groups.
    pub total_members_count: u32,
    pub group_last_id: GroupId,
    pub groups: UnorderedMap<GroupId, Group>,
    pub settings: LazyOption<VersionedSettings>,
    pub proposal_last_id: u32,
    pub proposals: UnorderedMap<u32, VersionedProposal>,
    pub storage: UnorderedMap<StorageKey, StorageBucket>,
    pub tags: UnorderedMap<TagCategory, Tags>,
    /// Provides metadata for dao actions.
    pub dao_action_metadata: LookupMap<DaoActionIdent, Vec<ObjectMetadata>>,
    pub function_call_metadata: UnorderedMap<FnCallId, Vec<ObjectMetadata>>,
    pub standard_function_call_metadata: UnorderedMap<MethodName, Vec<ObjectMetadata>>,
    pub workflow_last_id: u16,
    pub workflow_template: UnorderedMap<u16, (Template, Vec<TemplateSettings>)>,
    pub workflow_instance: UnorderedMap<ProposalId, Instance>,
    pub workflow_propose_settings: UnorderedMap<ProposalId, ProposeSettings>,
    /// Proposed workflow template settings for WorkflowAdd.
    pub proposed_workflow_settings: LookupMap<ProposalId, Vec<TemplateSettings>>,
    pub workflow_activity_log: LookupMap<ProposalId, Vec<ActivityLog>>, // Logs will be moved to indexer when its ready
    /// Id of last created treasury partition.
    pub partition_last_id: u16,
    pub treasury_partition: LookupMap<u16, VersionedTreasuryPartition>,
    /// Id of last created reward.
    pub reward_last_id: u16,
    pub rewards: LookupMap<u16, VersionedReward>,
    pub wallets: LookupMap<AccountId, VersionedWallet>,
    // TODO: Remove in production.
    pub debug_log: Vec<String>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        total_supply: u32,
        decimals: u8,
        settings: Settings,
        groups: Vec<GroupInput>,
        tags: Vec<TagInput>,
        standard_function_calls: Vec<MethodName>,
        standard_function_call_metadata: Vec<Vec<ObjectMetadata>>,
        function_calls: Vec<FnCallId>,
        function_call_metadata: Vec<Vec<ObjectMetadata>>,
        workflow_templates: Vec<Template>,
        workflow_template_settings: Vec<Vec<TemplateSettings>>,
        treasury_partitions: Vec<TreasuryPartitionInput>,
    ) -> Self {
        assert!(decimals <= 24);
        assert_valid_dao_settings(&settings);

        let mut contract = Contract {
            delegations: LookupMap::new(StorageKeys::Delegations),
            total_delegation_amount: 0,
            user_roles: LookupMap::new(StorageKeys::UserRoles),
            group_roles: LookupMap::new(StorageKeys::GroupRoles),
            ft_total_supply: total_supply,
            ft_total_locked: 0,
            ft_total_distributed: 0,
            total_members_count: 0,
            total_delegators_count: 0,
            decimals,
            settings: LazyOption::new(StorageKeys::DaoSettings, None),
            group_last_id: 0,
            groups: UnorderedMap::new(StorageKeys::Groups),
            proposal_last_id: 0,
            proposals: UnorderedMap::new(StorageKeys::Proposals),
            storage: UnorderedMap::new(StorageKeys::Storage),
            tags: UnorderedMap::new(StorageKeys::Tags),
            dao_action_metadata: LookupMap::new(StorageKeys::DaoActionMetadata),
            function_call_metadata: UnorderedMap::new(StorageKeys::FunctionCallMetadata),
            standard_function_call_metadata: UnorderedMap::new(
                StorageKeys::StandardFunctionCallMetadata,
            ),
            workflow_last_id: 0,
            workflow_template: UnorderedMap::new(StorageKeys::WfTemplate),
            workflow_instance: UnorderedMap::new(StorageKeys::WfInstance),
            workflow_propose_settings: UnorderedMap::new(StorageKeys::WfProposeSettings),

            proposed_workflow_settings: LookupMap::new(StorageKeys::ProposedWfTemplateSettings),
            workflow_activity_log: LookupMap::new(StorageKeys::ActivityLog),
            debug_log: Vec::default(),
            partition_last_id: 0,
            treasury_partition: LookupMap::new(StorageKeys::TreasuryPartition),
            reward_last_id: 0,
            rewards: LookupMap::new(StorageKeys::Rewards),
            wallets: LookupMap::new(StorageKeys::Wallet),
        };
        contract.init_dao_settings(settings);
        contract.init_tags(tags);
        contract.init_groups(groups);
        contract.init_function_calls(function_calls, function_call_metadata);
        contract
            .init_standard_function_calls(standard_function_calls, standard_function_call_metadata);
        contract.init_workflows(workflow_templates, workflow_template_settings);
        contract.storage_bucket_add(GLOBAL_BUCKET_IDENT);
        contract.init_treasury_partitions(treasury_partitions);

        contract
    }

    #[private]
    #[init(ignore_state)]
    pub fn upgrade() -> Self {
        assert!(env::storage_remove(
            &StorageKeys::NewVersionCode.into_storage_key()
        ));
        let mut _dao: Contract = env::state_read().expect("Failed to migrate");

        // ADD migration here

        _dao
    }

    /// For dev/testing purposes only
    #[cfg(feature = "testnet")]
    #[private]
    pub fn clean_self(&mut self) {
        self.workflow_template.clear();
        self.workflow_instance.clear();
        self.function_call_metadata.clear();

        self.proposals.clear();
        self.groups.clear();
        self.storage.clear();
        self.tags.clear();
        self.function_call_metadata.clear();
        self.workflow_template.clear();
        self.workflow_instance.clear();
        self.ft_metadata.remove();
        self.settings.remove();
    }

    /// For dev/testing purposes only
    #[cfg(feature = "testnet")]
    #[private]
    pub fn delete_self(&mut self) -> Promise {
        Promise::new(env::current_account_id()).delete_account("neardao.testnet".into())
    }
}

/// Triggers new version download from factory.
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn download_new_version() {
    use crate::constants::{GAS_DOWNLOAD_NEW_VERSION, VERSION};

    env::setup_panic_hook();

    // We are not able to access council members any other way so we have deserialize SC
    let contract: Contract = env::state_read().unwrap();
    let dao_settings: Settings = contract.settings.get().unwrap().into();

    //TODO download rights
    assert_eq!(
        dao_settings.dao_admin_account_id,
        env::predecessor_account_id()
    );

    let admin_acc = dao_settings.dao_admin_account_id;
    let method_name = "download_dao_bin";

    env::promise_create(
        admin_acc,
        method_name,
        &[VERSION],
        0,
        GAS_DOWNLOAD_NEW_VERSION,
    );
}

/// Method called by dao factory as response to download_new_version method.
/// Saves provided dao binary in storage under "NewVersionCode".
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn store_new_version() {
    env::setup_panic_hook();

    let contract: Contract = env::state_read().unwrap();
    let dao_settings: Settings = contract.settings.get().unwrap().into();

    assert_eq!(
        dao_settings.dao_admin_account_id,
        env::predecessor_account_id()
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
    use crate::constants::GAS_UPGRADE;

    env::setup_panic_hook();

    // We are not able to access council members any other way so we have deserialize SC
    let contract: Contract = env::state_read().unwrap();
    let dao_settings: Settings = contract.settings.get().unwrap().into();

    assert_eq!(
        dao_settings.dao_admin_account_id,
        env::predecessor_account_id()
    );

    let current_acc = env::current_account_id();
    let method_name = "upgrade";
    let key = StorageKeys::NewVersionCode.into_storage_key();

    let code = env::storage_read(key.as_slice()).expect("Failed to read code from storage.");
    let promise = env::promise_batch_create(&current_acc);
    env::promise_batch_action_deploy_contract(promise, code.as_slice());
    env::promise_batch_action_function_call(promise, method_name, &[], 0, GAS_UPGRADE);
}
