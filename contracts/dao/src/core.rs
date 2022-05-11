use crate::constants::{GLOBAL_BUCKET_IDENT, MAX_FT_TOTAL_SUPPLY};
use crate::event::{run_tick, Event, EventQueue};
use crate::internal::utils::current_timestamp_sec;
use crate::media::ResourceType;
use crate::reward::{RewardActivity, VersionedReward};
use crate::role::Role;
use crate::settings::{assert_valid_dao_settings, Settings, VersionedSettings};
use crate::tags::{TagInput, Tags};
use crate::treasury::TreasuryPartition;
use crate::wallet::VersionedWallet;
use library::storage::StorageBucket;
use library::types::datatype::Value;
use library::workflow::instance::{Instance, InstanceState};
use library::workflow::settings::{ProposeSettings, TemplateSettings};
use library::workflow::template::Template;
use library::workflow::types::{DaoActionIdent, ObjectMetadata};
use library::{FnCallId, MethodName};

use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_contract_standards::non_fungible_token::approval::NonFungibleTokenApprovalReceiver;
use near_contract_standards::non_fungible_token::core::NonFungibleTokenReceiver;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, near_bindgen, serde_json, AccountId, Balance, BorshStorageKey, IntoStorageKey,
    PanicOnDefault, PromiseOrValue,
};

use crate::group::{Group, GroupInput};

use crate::{proposal::*, DurationSec, RoleId, StorageKey, TagCategory, TimestampSec};
use crate::{GroupId, ProposalId};

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
// TODO: remove
pub struct ActivityLog {
    pub caller: AccountId,
    pub action_id: u8,
    pub timestamp: u64,
    pub args: Vec<Vec<Value>>,
    pub args_collections: Option<Vec<Vec<Value>>>,
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
    /// Vote token id.
    pub token_id: AccountId,
    /// Staking contract.
    pub staking_id: AccountId,
    /// Delegations per user.
    pub delegations: LookupMap<AccountId, Balance>,
    /// Delegated token total amount.
    pub total_delegation_amount: Balance,
    /// Event queues for ticks.
    pub events: LookupMap<TimestampSec, EventQueue<Event>>,
    /// Timestamp of the last fully processed tick queue.
    pub last_tick: TimestampSec,
    /// Time interval between two ticks.
    pub tick_interval: DurationSec,
    /// User's roles in groups.
    pub user_roles: LookupMap<AccountId, Vec<(GroupId, RoleId)>>,
    /// Group's provided roles.
    pub group_roles: LookupMap<GroupId, Role>,
    /// Total amount of minted tokens.
    pub ft_total_supply: u32,
    /// Decimals of token.
    pub decimals: u8,
    pub ft_total_locked: u32,
    pub ft_total_distributed: u32,
    /// Count of all members in groups - that does not mean unique members.
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
    // TODO: Remove in production.
    pub debug_log: Vec<String>,
    /// Id of last created treasury partition.
    pub partition_last_id: u16,
    pub treasury_partition: LookupMap<u16, TreasuryPartition>,
    /// Id of last created reward.
    pub reward_last_id: u16,
    pub rewards: LookupMap<u16, VersionedReward>,
    pub wallets: LookupMap<AccountId, VersionedWallet>,
}

#[near_bindgen]
impl Contract {
    #[allow(clippy::too_many_arguments)]
    #[init]
    pub fn new(
        token_id: AccountId,
        staking_id: AccountId,
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
        tick_interval: DurationSec,
    ) -> Self {
        assert!(total_supply <= MAX_FT_TOTAL_SUPPLY);
        assert_valid_dao_settings(&settings);

        let mut contract = Contract {
            token_id,
            staking_id,
            delegations: LookupMap::new(StorageKeys::Delegations),
            total_delegation_amount: 0,
            events: LookupMap::new(StorageKeys::Events),
            last_tick: 0,
            tick_interval,
            user_roles: LookupMap::new(StorageKeys::UserRoles),
            group_roles: LookupMap::new(StorageKeys::GroupRoles),
            ft_total_supply: total_supply,
            ft_total_locked: 0,
            ft_total_distributed: 0,
            total_members_count: 0,
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

    #[payable]
    pub fn proposal_create(
        &mut self,
        desc: ResourceType, // TODO: Optional
        template_id: u16,
        template_settings_id: u8,
        propose_settings: ProposeSettings,
        template_settings: Option<Vec<TemplateSettings>>,
    ) -> u32 {
        let caller = env::predecessor_account_id();
        let (wft, wfs) = self.workflow_template.get(&template_id).unwrap();
        let settings = wfs
            .get(template_settings_id as usize)
            .expect("Undefined settings id");
        assert!(env::attached_deposit() >= settings.deposit_propose.unwrap_or_else(|| 0.into()).0);
        if !self.check_rights(&settings.allowed_proposers, &caller) {
            panic!("You have no rights to propose this");
        }
        self.proposal_last_id += 1;
        //Assuming template_id for WorkflowAdd is always first wf added during dao init
        if template_id == 1 {
            assert!(
                template_settings.is_some(),
                "{}",
                "Expected template settings for 'WorkflowAdd' proposal"
            );
            self.proposed_workflow_settings
                .insert(&self.proposal_last_id, &template_settings.unwrap());
        }
        // TODO: Implement resource provider.
        let proposal = Proposal::new(
            0,
            env::block_timestamp() / 10u64.pow(9) / 60 * 60 + 60, // Rounded up to minutes
            caller,
            template_id,
            template_settings_id,
        );
        if wft.need_storage {
            if let Some(ref key) = propose_settings.storage_key {
                assert!(
                    self.storage.get(key).is_none(),
                    "Storage key already exists."
                );
            } else {
                panic!("Template requires storage, but no key was provided.");
            }
        }
        // TODO: Refactor.
        // Check that proposal binds have valid structure.
        //self.assert_valid_proposal_binds_structure(
        //    propose_settings.binds.as_slice(),
        //    wft.activities.as_slice(),
        //);
        self.proposals.insert(
            &self.proposal_last_id,
            &VersionedProposal::Current(proposal),
        );
        self.workflow_propose_settings
            .insert(&self.proposal_last_id, &propose_settings);
        // TODO: Croncat registration to finish proposal
        self.proposal_last_id
    }

    #[payable]
    pub fn proposal_vote(&mut self, id: u32, vote: u8) -> VoteResult {
        if vote > 2 {
            return VoteResult::InvalidVote;
        }
        let caller = env::predecessor_account_id();
        let (mut proposal, _, wfs) = self.get_workflow_and_proposal(id);
        assert!(
            env::attached_deposit() >= wfs.deposit_vote.unwrap_or_else(|| 0.into()).0,
            "{}",
            "Not enough deposit."
        );
        let TemplateSettings {
            allowed_voters,
            duration,
            vote_only_once,
            ..
        } = wfs;
        if !self.check_rights(&[allowed_voters], &caller) {
            return VoteResult::NoRights;
        }
        if proposal.state != ProposalState::InProgress
            || proposal.created + (duration as u64) < env::block_timestamp() / 10u64.pow(9)
        {
            return VoteResult::VoteEnded;
        }
        if vote_only_once && proposal.votes.contains_key(&caller) {
            return VoteResult::AlreadyVoted;
        }
        self.register_executed_activity(&caller, RewardActivity::Vote.into());
        proposal.votes.insert(caller, vote);
        self.proposals
            .insert(&id, &VersionedProposal::Current(proposal));
        VoteResult::Ok
    }

    pub fn proposal_finish(&mut self, id: u32) -> ProposalState {
        let caller = env::predecessor_account_id();
        let (mut proposal, wft, wfs) = self.get_workflow_and_proposal(id);
        let mut instance =
            Instance::new(proposal.workflow_id, wft.activities.len(), wft.end.clone());
        let propose_settings = self.workflow_propose_settings.get(&id).unwrap();
        let new_state = match proposal.state {
            ProposalState::InProgress => {
                if proposal.created + wfs.duration as u64 > env::block_timestamp() / 10u64.pow(9) {
                    None
                } else {
                    let vote_result = self.eval_votes(&proposal.votes, &wfs);
                    if matches!(vote_result, ProposalState::Accepted) {
                        instance.init_running(
                            wft.transitions.as_slice(),
                            wfs.transition_limits.as_slice(),
                        );
                        if let Some(ref storage_key) = propose_settings.storage_key {
                            self.storage_bucket_add(storage_key);
                        }
                    }
                    self.register_executed_activity(
                        &caller,
                        RewardActivity::AcceptedProposal.into(),
                    );
                    Some(vote_result)
                }
            }
            _ => None,
        };

        match new_state {
            Some(state) => {
                self.workflow_instance.insert(&id, &instance);
                proposal.state = state.clone();
                self.proposals
                    .insert(&id, &VersionedProposal::Current(proposal));

                if wft.auto_exec {
                    //TODO: Dispatch wf execution with Croncat.
                }

                state
            }
            None => proposal.state,
        }
    }

    // TODO: Implement autofinish on FatalError.
    /// Changes workflow instance state to finish.
    /// Rights to close are same as the "end" activity rights.
    pub fn wf_finish(&mut self, proposal_id: u32) -> bool {
        let caller = env::predecessor_account_id();
        let (proposal, wft, wfs) = self.get_workflow_and_proposal(proposal_id);

        assert!(proposal.state == ProposalState::Accepted);

        let mut wfi = self.workflow_instance.get(&proposal_id).unwrap();

        // TODO: Transition timestamp should not be included in this case.
        if wfi.get_state() == InstanceState::FatalError
            || self.check_rights(
                wfs.activity_rights[wfi.get_current_activity_id() as usize - 1].as_slice(),
                &caller,
            ) && wfi.new_actions_done(0, current_timestamp_sec())
        {
            self.workflow_instance.insert(&proposal_id, &wfi);
            true
        } else {
            false
        }
    }

    /// Unlocks FT for provided `GroupId`s by internal logic.
    /*     pub fn ft_unlock(&mut self, group_ids: Vec<GroupId>) -> Vec<u32> {
        let mut released = Vec::with_capacity(group_ids.len());
        for id in group_ids.into_iter() {
            if let Some(mut group) = self.groups.get(&id) {
                released.push(group.unlock_ft(env::block_timestamp() / 10u64.pow(9)));
                self.groups.insert(&id, &group);
            }
        }
        released
    } */

    /// Ticks and tries to process `count` of events in the last tick.
    /// Updates last_tick timestamp.
    /// Returns number of remaining events in last processed queue.
    /// DAO is supposed to tick when whenever possible.
    pub fn tick(&mut self, count: usize) -> usize {
        let current_timestamp = current_timestamp_sec();
        run_tick(self, count, current_timestamp)
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

#[derive(Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ReceiverMessage {
    pub proposal_id: u32,
}

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    /// TODO: Implement.
    /// TODO: Figure out how to assign storage keys.
    /// Required for some workflow scenarios.
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let msg: ReceiverMessage = serde_json::from_str(&msg).expect("invalid receiver msg");
        let prop_settings = self
            .workflow_propose_settings
            .get(&msg.proposal_id)
            .expect("proposal id does not exist");
        let storage_key = prop_settings
            .storage_key
            .expect("workflow does not have storage");
        let mut storage = self.storage.get(&storage_key).unwrap();
        storage.add_data(
            &"sender_id".to_string(),
            &Value::String(sender_id.to_string()),
        );
        storage.add_data(
            &"token_id".to_string(),
            &Value::String(env::predecessor_account_id().to_string()),
        );
        storage.add_data(&"amount".to_string(), &Value::U128(amount));
        self.storage.insert(&storage_key, &storage);
        PromiseOrValue::Value(U128(0))
    }
}

#[near_bindgen]
impl NonFungibleTokenReceiver for Contract {
    fn nft_on_transfer(
        &mut self,
        sender_id: AccountId,
        previous_owner_id: AccountId,
        token_id: near_contract_standards::non_fungible_token::TokenId,
        msg: String,
    ) -> PromiseOrValue<bool> {
        todo!()
    }
}

#[near_bindgen]
impl NonFungibleTokenApprovalReceiver for Contract {
    fn nft_on_approve(
        &mut self,
        token_id: near_contract_standards::non_fungible_token::TokenId,
        owner_id: AccountId,
        approval_id: u64,
        msg: String,
    ) -> near_sdk::PromiseOrValue<String> {
        todo!()
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
