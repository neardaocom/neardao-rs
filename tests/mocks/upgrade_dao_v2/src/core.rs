use crate::constants::GLOBAL_BUCKET_IDENT;
use crate::media::Media;
use crate::reward::VersionedReward;
use crate::role::{Roles, UserRoles};
use crate::settings::{assert_valid_dao_settings, Settings, VersionedSettings};
use crate::tags::{TagInput, Tags};
use crate::treasury::{TreasuryPartitionInput, VersionedTreasuryPartition};
use crate::wallet::VersionedWallet;
use library::storage::StorageBucket;
use library::types::Value;
use library::workflow::instance::Instance;
use library::workflow::settings::{ProposeSettings, TemplateSettings};
use library::workflow::template::Template;
use library::workflow::types::{DaoActionIdent, ObjectMetadata};
use library::{FnCallId, MethodName};

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, log, near_bindgen, AccountId, Balance, BorshStorageKey, IntoStorageKey, PanicOnDefault,
};

use crate::group::{Group, GroupInput};

use crate::{derive_from_versioned, derive_into_versioned, proposal::*, StorageKey, TagCategory};
use crate::{GroupId, ProposalId};

/// Action logs.
/// Will be removed when Indexer is ready.
#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct ActionLog {
    pub caller: AccountId,
    pub activity_id: u8,
    pub action_id: u8,
    pub timestamp_sec: u64,
    pub user_inputs: Vec<(String, Value)>,
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
    NewVersionUpgradeBin,
    NewVersionMigrationBin,
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
    Media,
    NonMigrableTestData,
    TestData,
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
    pub workflow_activity_log: LookupMap<ProposalId, Vec<ActionLog>>, // Logs will be moved to indexer when its ready
    /// Id of last created treasury partition.
    pub partition_last_id: u16,
    pub treasury_partition: LookupMap<u16, VersionedTreasuryPartition>,
    /// Id of last created reward.
    pub reward_last_id: u16,
    pub rewards: LookupMap<u16, VersionedReward>,
    pub wallets: LookupMap<AccountId, VersionedWallet>,
    // TODO: Remove in production.
    pub debug_log: Vec<String>,
    pub media_last_id: u32,
    pub media: LookupMap<u32, Media>,
    pub test_data: UnorderedSet<VersionedTestData>,
    pub non_migrable_test_data: UnorderedSet<VersionedNonMigrableTestData>,
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
        media: Vec<Media>,
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
            media_last_id: 0,
            media: LookupMap::new(StorageKeys::Media),
            test_data: UnorderedSet::new(StorageKeys::TestData),
            non_migrable_test_data: UnorderedSet::new(StorageKeys::NonMigrableTestData),
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
        contract.init_media(media);
        contract
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

    #[private]
    #[init(ignore_state)]
    pub fn deploy_upgrade_bin() -> Self {
        let contract: Contract = env::state_read().unwrap();
        log!("{:?}", contract.test_data.to_vec());
        log!("{:?}", contract.non_migrable_test_data.to_vec());
        env::storage_remove(&StorageKeys::NewVersionMigrationBin.into_storage_key());
        env::storage_remove(&StorageKeys::NewVersionUpgradeBin.into_storage_key());
        contract
    }

    pub fn add_dummy_data(&mut self, data: Vec<String>) {
        for d in data.into_iter() {
            self.test_data.insert(
                &TestData {
                    string_data: d.clone(),
                    new_string_data: d.clone(),
                }
                .into(),
            );
            self.non_migrable_test_data.insert(
                &NonMigrableTestDataNew {
                    string_data: d.clone(),
                    new_string_data: d.clone(),
                }
                .into(),
            );
        }
    }

    pub fn view_dummy_data(&self) -> (Vec<TestData>, Vec<VersionedNonMigrableTestData>) {
        let test_data = self
            .test_data
            .to_vec()
            .into_iter()
            .map(|e| e.into())
            .collect();
        let non_migrable = self.non_migrable_test_data.to_vec();
        (test_data, non_migrable)
    }
}

derive_from_versioned!(VersionedTestData, TestData, New);
derive_into_versioned!(TestData, VersionedTestData, New);

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum VersionedTestData {
    First,
    New(TestData),
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct TestData {
    string_data: String,
    new_string_data: String,
}

derive_from_versioned!(VersionedNonMigrableTestData, NonMigrableTestDataNew, New);
derive_into_versioned!(NonMigrableTestDataNew, VersionedNonMigrableTestData, New);

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum VersionedNonMigrableTestData {
    Current(NonMigrableTestData),
    New(NonMigrableTestDataNew),
}
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct NonMigrableTestData {
    string_data: String,
    //garbage: String,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct NonMigrableTestDataNew {
    string_data: String,
    new_string_data: String,
}
