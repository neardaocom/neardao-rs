use crate::constants::{GLOBAL_BUCKET_IDENT, MAX_FT_TOTAL_SUPPLY};
use crate::event::{run_tick, Event, EventQueue};
use crate::internal::utils::current_timestamp_sec;
use crate::role::Role;
use crate::settings::{assert_valid_dao_settings, DaoSettings, VDaoSettings};
use crate::tags::{TagInput, Tags};
use library::storage::StorageBucket;
use library::types::datatype::Value;
use library::workflow::instance::{Instance, InstanceState};
use library::workflow::settings::{ProposeSettings, TemplateSettings};
use library::workflow::template::Template;
use library::workflow::types::{DaoActionIdent, ObjectMetadata};
use library::{FnCallId, MethodName};

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap};
use near_sdk::serde::Serialize;
use near_sdk::{
    env, log, near_bindgen, AccountId, Balance, BorshStorageKey, IntoStorageKey, PanicOnDefault,
};

use crate::group::{Group, GroupInput};

use crate::{
    calc_percent_u128_unchecked, proposal::*, DurationSec, RoleId, StorageKey, TagCategory,
    TimestampSec,
};
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
    ProposedWfTemplateSettings,
    ActivityLog,
    WfInstance,
    DaoActionMetadata,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
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
    /// Interval between ticks.
    pub tick_interval: DurationSec,
    /// User's roles.
    pub user_roles: LookupMap<AccountId, Vec<(GroupId, RoleId)>>,
    /// Group's roles.
    pub group_roles: LookupMap<GroupId, Role>,
    pub ft_total_supply: u32,
    pub ft_total_locked: u32,
    pub ft_total_distributed: u32,
    /// Count of all members in groups - that does not mean unique members.
    pub total_members_count: u32,
    pub decimal_const: u128,
    pub group_last_id: GroupId,
    pub groups: UnorderedMap<GroupId, Group>,
    pub settings: LazyOption<VDaoSettings>,
    pub proposal_last_id: u32,
    pub proposals: UnorderedMap<u32, VProposal>,
    pub storage: UnorderedMap<StorageKey, StorageBucket>,
    pub tags: UnorderedMap<TagCategory, Tags>,
    /// Provides metadata for dao actions.
    pub dao_action_metadata: LookupMap<DaoActionIdent, Vec<ObjectMetadata>>,
    pub function_call_metadata: UnorderedMap<FnCallId, Vec<ObjectMetadata>>,
    pub standard_function_call_metadata: UnorderedMap<MethodName, Vec<ObjectMetadata>>,
    pub workflow_last_id: u16,
    pub workflow_template: UnorderedMap<u16, (Template, Vec<TemplateSettings>)>,
    pub workflow_instance: UnorderedMap<ProposalId, (Instance, ProposeSettings)>,
    /// Proposed workflow template settings for WorkflowAdd.
    pub proposed_workflow_settings: LookupMap<ProposalId, Vec<TemplateSettings>>,
    pub workflow_activity_log: LookupMap<ProposalId, Vec<ActivityLog>>, // Logs will be moved to indexer when its ready
}

#[near_bindgen]
impl Contract {
    #[allow(clippy::too_many_arguments)]
    #[init]
    pub fn new(
        staking_id: AccountId,
        total_supply: u32,
        settings: DaoSettings,
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
            decimal_const: 10u128.pow(24), // TODO
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
            proposed_workflow_settings: LookupMap::new(StorageKeys::ProposedWfTemplateSettings),
            workflow_activity_log: LookupMap::new(StorageKeys::ActivityLog),
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
        desc: String,
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

        let proposal = Proposal::new(
            desc,
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

        // Check that proposal binds have valid structure.
        self.assert_valid_proposal_binds_structure(
            propose_settings.binds.as_slice(),
            wft.activities.as_slice(),
        );

        self.proposals
            .insert(&self.proposal_last_id, &VProposal::Curr(proposal));
        self.workflow_instance.insert(
            &self.proposal_last_id,
            &(Instance::new(template_id), propose_settings),
        );

        // TODO: Croncat registration to finish proposal

        self.proposal_last_id
    }

    #[payable]
    pub fn proposal_vote(&mut self, proposal_id: u32, vote_kind: u8) -> VoteResult {
        if vote_kind > 2 {
            return VoteResult::InvalidVote;
        }

        let caller = env::predecessor_account_id();
        let (mut proposal, _, wfs) = self.get_workflow_and_proposal(proposal_id);

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

        proposal.votes.insert(caller, vote_kind);

        self.proposals
            .insert(&proposal_id, &VProposal::Curr(proposal));

        VoteResult::Ok
    }

    pub fn finish_proposal(&mut self, proposal_id: u32) -> ProposalState {
        let (mut proposal, wft, wfs) = self.get_workflow_and_proposal(proposal_id);
        let (mut instance, settings) = self.workflow_instance.get(&proposal_id).unwrap();

        let new_state = match proposal.state {
            ProposalState::InProgress => {
                if proposal.created + wfs.duration as u64 > env::block_timestamp() / 10u64.pow(9) {
                    None
                } else {
                    // count votes
                    let (max_possible_amount, vote_results) =
                        self.calculate_votes(&proposal.votes, &wfs.scenario, &wfs.allowed_voters);
                    log!("Votes: {}, {:?}", max_possible_amount, vote_results);
                    // check spam
                    if calc_percent_u128_unchecked(
                        vote_results[0],
                        max_possible_amount,
                        self.decimal_const,
                    ) >= wfs.spam_threshold
                    {
                        Some(ProposalState::Spam)
                    } else if calc_percent_u128_unchecked(
                        vote_results.iter().sum(),
                        max_possible_amount,
                        self.decimal_const,
                    ) < wfs.quorum
                    {
                        Some(ProposalState::Invalid)
                    } else if calc_percent_u128_unchecked(
                        vote_results[1],
                        vote_results.iter().sum(),
                        self.decimal_const,
                    ) < wfs.approve_threshold
                    {
                        Some(ProposalState::Rejected)
                    } else {
                        instance.state = InstanceState::Running;
                        instance.init_transition_counter(
                            self.create_transition_counter(&wft.transitions),
                        );

                        if let Some(ref storage_key) = settings.storage_key {
                            self.storage_bucket_add(storage_key);
                        }
                        Some(ProposalState::Accepted)
                    }
                }
            }
            _ => None,
        };

        match new_state {
            Some(state) => {
                self.workflow_instance
                    .insert(&proposal_id, &(instance, settings));
                proposal.state = state.clone();
                self.proposals
                    .insert(&proposal_id, &VProposal::Curr(proposal));

                if wft.is_simple {
                    //TODO: Dispatch wf execution with Croncat.
                }

                state
            }
            None => proposal.state,
        }
    }

    /// Changes workflow instance state to finish.
    /// Rights to close are same as the "end" activity rights.
    pub fn wf_finish(&mut self, proposal_id: u32) -> bool {
        let caller = env::predecessor_account_id();
        let (proposal, wft, wfs) = self.get_workflow_and_proposal(proposal_id);

        assert!(proposal.state == ProposalState::Accepted);

        let (mut wfi, settings) = self.workflow_instance.get(&proposal_id).unwrap();

        if wfi.state == InstanceState::FatalError
            || self.check_rights(
                wfs.activity_rights[wfi.current_activity_id as usize - 1].as_slice(),
                &caller,
            ) && wft.end.contains(&wfi.current_activity_id)
        {
            wfi.state = InstanceState::Finished;
            self.workflow_instance
                .insert(&proposal_id, &(wfi, settings));
            true
        } else {
            false
        }
    }

    /// Unlocks FT for provided `GroupId`s by internal logic.
    pub fn ft_unlock(&mut self, group_ids: Vec<GroupId>) -> Vec<u32> {
        let mut released = Vec::with_capacity(group_ids.len());
        for id in group_ids.into_iter() {
            if let Some(mut group) = self.groups.get(&id) {
                released.push(group.unlock_ft(env::block_timestamp() / 10u64.pow(9)));
                self.groups.insert(&id, &group);
            }
        }
        released
    }

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

/// Triggers new version download from factory.
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn download_new_version() {
    use crate::constants::{GAS_DOWNLOAD_NEW_VERSION, VERSION};

    env::setup_panic_hook();

    // We are not able to access council members any other way so we have deserialize SC
    let contract: Contract = env::state_read().unwrap();
    let dao_settings: DaoSettings = contract.settings.get().unwrap().into();

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
    let dao_settings: DaoSettings = contract.settings.get().unwrap().into();

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
    let dao_settings: DaoSettings = contract.settings.get().unwrap().into();

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
