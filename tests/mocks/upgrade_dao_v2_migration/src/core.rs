#![allow(clippy::too_many_arguments)]

use crate::constants::GLOBAL_BUCKET_IDENT;
use crate::media::Media;
use crate::reward::VersionedReward;
use crate::role::{Roles, UserRoles};
use crate::settings::{Settings, VersionedSettings};
use crate::tags::{TagInput, Tags};
use crate::treasury::{TreasuryPartitionInput, VersionedTreasuryPartition};
use crate::wallet::VersionedWallet;
use library::storage::StorageBucket;
use library::types::datatype::Value;
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
    #[private]
    #[init(ignore_state)]
    pub fn deploy_migration_bin() -> Self {
        assert!(
            env::storage_has_key(&StorageKeys::NewVersionUpgradeBin.into_storage_key()),
            "missing upgrade dao bin"
        );
        let contract: Contract = env::state_read().unwrap();
        contract
    }

    pub fn migrate_data(&mut self) {
        let data: Vec<TestData> = self
            .test_data
            .to_vec()
            .into_iter()
            .map(|e| match e {
                VersionedTestData::New(_) => unreachable!(),
                VersionedTestData::Current(d) => d,
            })
            .collect();
        for d in data {
            self.test_data
                .remove(&VersionedTestData::Current(d.clone()));
            let new = TestDataNew {
                string_data: d.string_data,
                new_string_data: "migrated".into(),
                //new_string_data: d.string_data,
            };
            self.test_data.insert(&VersionedTestData::New(new));
        }
        log!("{:?}", self.test_data.to_vec());
        log!("{:?}", self.non_migrable_test_data.to_vec());
        assert!(
            env::storage_remove(&StorageKeys::NewVersionMigrationBin.into_storage_key()),
            "internal - missing migration bin"
        );
    }
}

//derive_from_versioned!(VersionedTestData, TestData);
//derive_into_versioned!(TestData, VersionedTestData);

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum VersionedTestData {
    Current(TestData),
    New(TestDataNew),
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct TestData {
    string_data: String,
    //garbage: String,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct TestDataNew {
    string_data: String,
    new_string_data: String,
}

//derive_from_versioned!(VersionedNonMigrableTestData, NonMigrableTestDataNew);
//derive_into_versioned!(NonMigrableTestDataNew, VersionedNonMigrableTestData);

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
