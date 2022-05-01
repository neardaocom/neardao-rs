use std::{collections::HashMap, convert::TryInto};

use near_sdk::{
    env::{self},
    require, AccountId, Balance, Promise,
};

use crate::{
    constants::{C_DAO_ACC_ID, GLOBAL_BUCKET_IDENT, TGAS},
    core::{ActivityLog, Contract},
    error::{
        ActionError, ERR_DISTRIBUTION_NOT_ENOUGH_FT, ERR_GROUP_HAS_NO_LEADER, ERR_GROUP_NOT_FOUND,
        ERR_LOCK_AMOUNT_OVERFLOW, ERR_STORAGE_BUCKET_EXISTS,
    },
    event::{Event, EventQueue},
    group::{Group, GroupInput},
    internal::utils::current_timestamp_sec,
    proposal::Proposal,
    settings::DaoSettings,
    tags::{TagInput, Tags},
    token_lock::TokenLock,
    CalculatedVoteResults, ProposalId, ProposalWf, TimestampSec, VoteTotalPossible, Votes,
};
use library::{
    functions::binding::get_value_from_source,
    storage::StorageBucket,
    types::{
        activity_input::ActivityInput, consts::Consts, datatype::Value, error::ProcessingError,
        source::Source,
    },
    workflow::{
        action::{ActionInput, FnCallIdType, TemplateAction},
        activity::{Activity, Terminality, Transition},
        instance::InstanceState,
        postprocessing::Postprocessing,
        settings::{ActivityBind, ProposeSettings, TemplateSettings},
        template::Template,
        types::{ActivityRight, DaoActionIdent, ObjectMetadata, VoteScenario},
    },
    FnCallId, MethodName,
};

impl Contract {
    #[inline]
    pub fn init_dao_settings(&mut self, settings: DaoSettings) {
        self.settings.set(&settings.into());
    }

    #[inline]
    pub fn init_tags(&mut self, tags: Vec<TagInput>) {
        self.tags.insert(&"group".into(), &Tags::new());
        self.tags.insert(&"media".into(), &Tags::new());

        for i in tags.into_iter() {
            let mut tags = Tags::new();
            tags.insert(i.values);
            self.tags.insert(&i.category, &tags);
        }
    }

    #[inline]
    pub fn init_groups(&mut self, groups: Vec<GroupInput>) {
        for g in groups.into_iter() {
            self.add_group(g);
        }

        assert!(
            self.ft_total_supply >= self.ft_total_locked,
            "{}",
            ERR_LOCK_AMOUNT_OVERFLOW
        );
    }

    /// Registers fncalls and their metadata.
    /// Existing are overwriten.
    /// No checks included.
    pub fn init_function_calls(
        &mut self,
        calls: Vec<FnCallId>,
        metadata: Vec<Vec<ObjectMetadata>>,
    ) {
        for (i, c) in calls.iter().enumerate() {
            self.function_call_metadata.insert(c, &metadata[i]);
        }
    }

    /// Version of `init_function_calls` method but for standard interfaces.
    pub fn init_standard_function_calls(
        &mut self,
        calls: Vec<MethodName>,
        metadata: Vec<Vec<ObjectMetadata>>,
    ) {
        for (i, c) in calls.iter().enumerate() {
            self.standard_function_call_metadata.insert(c, &metadata[i]);
        }
    }

    // Each workflow must have at least one setting
    #[inline]
    pub fn init_workflows(
        &mut self,
        mut workflows: Vec<Template>,
        mut workflow_template_settings: Vec<Vec<TemplateSettings>>,
    ) {
        //assert!(workflows.len() > 0);
        //assert!(
        //    workflows[0]
        //        .get_activity_as_ref(1)
        //        .unwrap()
        //        .get_dao_action_type(0)
        //        .unwrap()
        //        == DaoActionIdent::WorkflowAdd,
        //    "{}",
        //    "First Workflow must be WorkflowAdd"
        //);

        let len = workflows.len();
        for i in 0..len {
            self.workflow_template.insert(
                &((len - i) as u16),
                &(
                    workflows.pop().unwrap(),
                    workflow_template_settings.pop().unwrap(),
                ),
            );
        }

        self.workflow_last_id += len as u16;
    }

    pub fn get_workflow_and_proposal(&self, proposal_id: u32) -> ProposalWf {
        let proposal = Proposal::from(self.proposals.get(&proposal_id).expect("Unknown proposal"));
        let (wft, mut wfs) = self.workflow_template.get(&proposal.workflow_id).unwrap();
        let settings = wfs.swap_remove(proposal.workflow_settings_id as usize);

        (proposal, wft, settings)
    }

    // TODO: Unit tests
    pub fn check_rights(&self, rights: &[ActivityRight], account_id: &AccountId) -> bool {
        if rights.is_empty() {
            return true;
        }

        for right in rights.iter() {
            match right {
                ActivityRight::Anyone => {
                    return true;
                }
                ActivityRight::Group(g) => match self.groups.get(g) {
                    Some(group) => match group.get_member_by_account(account_id) {
                        Some(_m) => return true,
                        None => continue,
                    },
                    _ => panic!("{}", ERR_GROUP_NOT_FOUND),
                },
                ActivityRight::GroupMember(g, name) => {
                    if *name != *account_id {
                        continue;
                    }

                    match self.groups.get(g) {
                        Some(group) => match group.get_member_by_account(account_id) {
                            Some(_m) => return true,
                            None => continue,
                        },
                        _ => panic!("{}", ERR_GROUP_NOT_FOUND),
                    }
                }
                ActivityRight::TokenHolder => unimplemented!(),
                ActivityRight::GroupRole(g, r) => match self.groups.get(g) {
                    Some(group) => match group.get_member_by_account(account_id) {
                        Some(m) => match m.tags.into_iter().any(|t| t == *r) {
                            true => return true,
                            false => continue,
                        },
                        None => continue,
                    },
                    _ => panic!("{}", ERR_GROUP_NOT_FOUND),
                },
                ActivityRight::GroupLeader(g) => match self.groups.get(g) {
                    Some(group) => {
                        if let Some(leader) = group.settings.leader {
                            match leader == *account_id {
                                true => return true,
                                false => continue,
                            }
                        } else {
                            panic!("{}", ERR_GROUP_HAS_NO_LEADER);
                        }
                    }
                    _ => panic!("{}", ERR_GROUP_NOT_FOUND),
                },
                //TODO only group members
                ActivityRight::Member => unimplemented!(),
                ActivityRight::Account(a) => match a == account_id {
                    true => return true,
                    false => continue,
                },
            }
        }
        false
    }

    /// Evaluates vote results by scenario and type of voters.
    /// Returns tuple (max_possible_amount,vote_results)
    #[allow(unused)]
    pub fn calculate_votes(
        &self,
        votes: &HashMap<AccountId, u8>,
        scenario: &VoteScenario,
        vote_target: &ActivityRight,
    ) -> CalculatedVoteResults {
        let mut vote_result: Votes = [0_u128; 3];
        let mut max_possible_amount: VoteTotalPossible = 0;
        match scenario {
            VoteScenario::Democratic => {
                match vote_target {
                    ActivityRight::Anyone => {
                        max_possible_amount = votes.len() as u128;
                    }
                    ActivityRight::Group(g) => match self.groups.get(g) {
                        Some(group) => {
                            max_possible_amount = group.members.members_count() as u128;
                        }
                        None => panic!("{}", ERR_GROUP_NOT_FOUND),
                    },
                    ActivityRight::GroupMember(_, _)
                    | ActivityRight::Account(_)
                    | ActivityRight::GroupLeader(_) => {
                        max_possible_amount = 1;
                    }
                    ActivityRight::TokenHolder => {
                        unimplemented!()
                    }
                    // If member exists in 2 groups, then he is accounted twice.
                    ActivityRight::Member => {
                        max_possible_amount = self.total_members_count as u128;
                    }
                    ActivityRight::GroupRole(g, r) => match self.groups.get(g) {
                        Some(group) => {
                            max_possible_amount =
                                group.get_members_accounts_by_role(*r).len() as u128;
                        }
                        None => panic!("{}", ERR_GROUP_NOT_FOUND),
                    },
                }

                for vote_value in votes.values() {
                    vote_result[*vote_value as usize] += 1;
                }
            }
            VoteScenario::TokenWeighted => match vote_target {
                ActivityRight::Anyone | ActivityRight::TokenHolder => unimplemented!(),
                // This is expensive scenario
                ActivityRight::Member => unimplemented!(),
                ActivityRight::Group(gid) => unimplemented!(),
                ActivityRight::GroupRole(gid, rid) => unimplemented!(),
                ActivityRight::GroupMember(_, _)
                | ActivityRight::Account(_)
                | ActivityRight::GroupLeader(_) => {
                    max_possible_amount = 1;
                    for vote_value in votes.values() {
                        vote_result[*vote_value as usize] += 1;
                    }
                }
            },
        }

        (max_possible_amount, vote_result)
    }

    pub fn storage_bucket_add(&mut self, bucket_id: &str) {
        let bucket = StorageBucket::new(utils::get_bucket_id(bucket_id));
        assert!(
            self.storage
                .insert(&bucket_id.to_owned(), &bucket)
                .is_none(),
            "{}",
            ERR_STORAGE_BUCKET_EXISTS
        );
    }

    /// Adds new Group with TokenLock.
    /// Updates DAO's `ft_total_locked` amount and `total_members_count` values.
    pub fn add_group(&mut self, group: GroupInput) {
        // Already counted members still counts as new.
        self.total_members_count += group.members.len() as u32;

        let token_lock = if let Some(tl) = group.token_lock {
            self.ft_total_locked += tl.amount;

            // Check if dao has enough free tokens to distribute ft
            if tl.init_distribution > 0 {
                assert!(
                    tl.init_distribution
                        <= self.ft_total_supply - self.ft_total_locked - self.ft_total_distributed,
                    "{}",
                    ERR_DISTRIBUTION_NOT_ENOUGH_FT
                );
                self.distribute_ft(
                    tl.init_distribution,
                    &group
                        .members
                        .iter()
                        .map(|member| member.account_id.clone())
                        .collect::<Vec<AccountId>>(), //TODO optimalize
                );
            }

            // TODO: Should return Err<T>
            let tl: TokenLock = tl.try_into().expect("Failed to create TokenLock.");
            Some(tl)
        } else {
            None
        };

        // Create StorageKey for nested structure
        self.group_last_id += 1;
        let token_lock_key = utils::get_group_key(self.group_last_id);

        self.groups.insert(
            &self.group_last_id,
            &Group::new(token_lock_key, group.settings, group.members, token_lock),
        );
    }

    /// Internally transfers FT from contract account all accounts equally.
    /// Sets contract's ft_total_distributed property.
    /// Panics if account_ids is empty vector or distribution value is zero.
    #[allow(unused)]
    pub fn distribute_ft(&mut self, amount: u32, account_ids: &[AccountId]) {
        unimplemented!("Requires new FT SC implemented.");
        /*         assert!(!account_ids.is_empty(), "{}", ERR_DISTRIBUTION_ACC_EMPTY);
        assert!(
            amount / account_ids.len() as u32 > 0,
            "{}",
            ERR_DISTRIBUTION_MIN_VALUE
        );
        let amount_per_acc = (amount / account_ids.len() as u32) as u128 * self.decimal_const;
        self.ft_total_distributed += amount - (amount % account_ids.len() as u32);
        let contract_account_id = env::current_account_id();
        for acc in account_ids {
            // If not registered when distributing ft, we register them, assuming storage deposit payment is solved by other mechanisms
            if !self.ft.accounts.contains_key(acc) {
                self.ft.accounts.insert(acc, &0);
            }

            self.ft
                .internal_transfer(&contract_account_id, acc, amount_per_acc, None);
        } */
    }

    /// Error callback.
    /// If promise did not have to succeed, then instance is still updated.
    pub fn postprocessing_failed(&mut self, proposal_id: u32, must_succeed: bool) {
        let (mut wfi, settings) = self.workflow_instance.get(&proposal_id).unwrap();

        let awaiting_state = wfi.awaiting_state.take().unwrap();

        if must_succeed {
            // Switch state back to running.
            wfi.unset_awaiting_state(InstanceState::Running);

            // TODO: Question is if to do postprocessing as well or just update instance.
        } else {
            if awaiting_state.is_new_transition {
                wfi.transition_next(
                    awaiting_state.activity_id,
                    awaiting_state.new_activity_actions_count,
                    1,
                );
            } else {
                wfi.actions_done_count += 1;
            }

            // This might be unnecessary as activity with one optional action does not make much sense.
            if wfi.actions_done_count == wfi.actions_total && awaiting_state.wf_finish {
                wfi.unset_awaiting_state(InstanceState::Finished);
            } else {
                wfi.awaiting_state = Some(awaiting_state);
            }
        }

        self.workflow_instance
            .insert(&proposal_id, &(wfi, settings));
    }

    /// Success callback.
    /// Modifies workflow's instance.
    /// If `postprocessing` is included, then also postprocessing script is executed.
    /// Only successful postprocessing updates action as sucessfully executed.
    pub fn postprocessing_success(
        &mut self,
        proposal_id: u32,
        action_id: u8,
        storage_key: Option<String>,
        postprocessing: Option<Postprocessing>,
        promise_call_result: Vec<u8>,
    ) {
        let (mut wfi, settings) = self.workflow_instance.get(&proposal_id).unwrap();

        let awaiting_state = wfi.awaiting_state.take().unwrap();

        // Action transaction check if previous action succesfully finished.
        if wfi.actions_done_count < action_id {
            wfi.unset_awaiting_state(InstanceState::Running);
            self.workflow_instance
                .insert(&proposal_id, &(wfi, settings));
            return;
        }
        wfi.last_transition_done_at = env::block_timestamp();
        wfi.actions_done_count += 1;

        // Check if its last action done.
        if wfi.actions_done_count == wfi.actions_total {
            if awaiting_state.wf_finish {
                wfi.unset_awaiting_state(InstanceState::Finished);
            } else {
                wfi.unset_awaiting_state(InstanceState::Running);
            }
        // Check if its first action done
        } else if wfi.actions_done_count == 1 {
            wfi.transition_next(
                awaiting_state.activity_id,
                awaiting_state.new_activity_actions_count,
                1,
            );
        }

        // Execute postprocessing script which must always succeed.
        if let Some(pp) = postprocessing {
            let mut global_storage = self.storage.get(&GLOBAL_BUCKET_IDENT.into()).unwrap();
            let mut storage = if let Some(ref storage_key) = storage_key {
                self.storage.get(storage_key)
            } else {
                None
            };

            let mut new_template = None;

            if pp
                .execute(
                    promise_call_result,
                    storage.as_mut(),
                    &mut global_storage,
                    &mut new_template,
                )
                .is_err()
            {
                wfi.unset_awaiting_state(InstanceState::FatalError);
            } else {
                // Only in case its workflow Add.
                if let Some((
                    workflow,
                    fncalls,
                    fncall_metadata,
                    std_fncalls,
                    std_fncall_metadata,
                )) = new_template
                {
                    // Unwraping is ok as settings are inserted when this proposal is accepted.
                    let settings = self
                        .proposed_workflow_settings
                        .remove(&proposal_id)
                        .unwrap();

                    self.workflow_last_id += 1;
                    self.workflow_template
                        .insert(&self.workflow_last_id, &(workflow, settings));
                    self.init_function_calls(fncalls, fncall_metadata);
                    self.init_standard_function_calls(std_fncalls, std_fncall_metadata);
                }

                // Save updated storages.
                if let Some(storage) = storage {
                    self.storage.insert(&storage_key.unwrap(), &storage);
                }
                self.storage
                    .insert(&GLOBAL_BUCKET_IDENT.into(), &global_storage);
            }
        };

        wfi.awaiting_state = Some(awaiting_state);
        self.workflow_instance
            .insert(&proposal_id, &(wfi, settings));
    }

    /// Closure which might be required in workflow.
    /// Returns DAO's specific values which cannot be known ahead of time.
    pub fn dao_consts(&self) -> impl Consts {
        DaoConsts::default()
    }

    /// Action logging method.
    /// Will be moved to indexer when its ready.
    pub fn log_action(
        &mut self,
        proposal_id: ProposalId,
        caller: &AccountId,
        action_id: u8,
        args: &[Vec<Value>],
        args_collections: Option<&[Vec<Value>]>,
    ) {
        let mut logs = self
            .workflow_activity_log
            .get(&proposal_id)
            .unwrap_or_else(|| Vec::with_capacity(1));

        logs.push(ActivityLog {
            caller: caller.to_owned(),
            action_id,
            timestamp: env::block_timestamp() / 10u64.pow(9),
            args: args.to_vec(),
            args_collections: args_collections.map(|a| a.to_vec()),
        });

        self.workflow_activity_log.insert(&proposal_id, &logs);
    }

    /// Creates transition counter for `Instance`
    pub fn create_transition_counter(&self, transitions: &[Vec<Transition>]) -> Vec<Vec<u16>> {
        let mut counter = Vec::with_capacity(transitions.len());

        for t in transitions {
            counter.push(vec![0; t.len()]);
        }

        counter
    }

    /// Checks if inputs structure is same as activity definition.
    /// Same order as activity's actions is required.
    /// Done activities are not required to be in inputs.
    /// Skips done actions.
    pub fn check_activity_input(
        &self,
        actions: &[TemplateAction],
        inputs: &[Option<ActionInput>],
        actions_done: usize,
    ) -> bool {
        for (idx, action) in actions.iter().enumerate().skip(actions_done) {
            match (
                action.optional,
                inputs
                    .get(idx - actions_done)
                    .expect("Missing action input"),
            ) {
                (_, Some(a)) => {
                    if !a.action.eq(&action.action_data) {
                        return false;
                    }
                }
                (false, None) => return false,
                _ => continue,
            }
        }

        true
    }

    // TODO: refactor
    /// Executes DAO's native action.
    /// Inner methods panic when provided malformed inputs - structure/datatype.
    pub fn execute_dao_action(
        &mut self,
        _proposal_id: u32,
        action_ident: DaoActionIdent,
        inputs: &mut dyn ActivityInput,
    ) -> Result<(), ActionError> {
        match action_ident {
            /*             DaoActionIdent::GroupAdd => {
                let group_input = deserialize_group_input(inputs)?;
                self.group_add(group_input);
            }
            DaoActionIdent::GroupRemove => {
                self.group_remove(get_datatype_from_values(inputs, 0, 0)?.try_into_u64()? as u16);
            }
            DaoActionIdent::GroupUpdate => {
                let group_id = get_datatype_from_values(inputs, 0, 0)?.try_into_u64()? as u16;
                let group_settings = deserialize_group_settings(inputs, 1)?;
                self.group_update(group_id, group_settings);
            }
            DaoActionIdent::GroupAddMembers => {
                let group_id = get_datatype_from_values(inputs, 0, 0)?.try_into_u64()? as u16;
                let group_members = deserialize_group_members(inputs, 1)?;
                self.group_add_members(group_id, group_members);
            }
            DaoActionIdent::GroupRemoveMember => {
                let member = get_datatype_from_values(inputs, 0, 1)?
                    .try_into_string()?
                    .try_into()
                    .map_err(|_| ActionError::Binding)?;

                self.group_remove_member(
                    get_datatype_from_values(inputs, 0, 0)?.try_into_u64()? as u16,
                    member,
                );
            }
            DaoActionIdent::SettingsUpdate => {
                let settings_input = deserialize_dao_settings(inputs)?;
                self.settings_update(settings_input);
            }
            DaoActionIdent::TagAdd => unimplemented!(),
            DaoActionIdent::TagEdit => unimplemented!(),
            DaoActionIdent::TagRemove => unimplemented!(),
            DaoActionIdent::FtDistribute => {
                let (group_id, amount, account_ids) = (
                    get_datatype_from_values(inputs, 0, 0)?.try_into_u64()? as u16,
                    get_datatype_from_values(inputs, 0, 1)?.try_into_u64()? as u32,
                    get_datatype_from_values(inputs, 0, 2)?.try_into_vec_string()?,
                );

                let mut accounts = Vec::with_capacity(account_ids.len());
                for acc in account_ids.into_iter() {
                    accounts.push(acc.try_into().map_err(|_| ActionError::Binding)?);
                }

                self.ft_distribute(group_id, amount, accounts);
            } */
            _ => unreachable!(),
        }

        Ok(())
    }

    pub fn execute_fn_call_action(
        &mut self,
        mut receiver: AccountId,
        method: String,
        inputs: &[Vec<Value>],
        deposit: u128,
        tgas: u16,
        metadata: &[ObjectMetadata],
    ) -> Promise {
        if receiver.as_str() == "self" {
            receiver = env::current_account_id();
        }

        //let args = serialize_to_json(inputs, metadata, 0);
        let args = "".to_string();

        Promise::new(receiver).function_call(method, args.into_bytes(), deposit, TGAS * tgas as u64)
    }

    /// Proposal binds structure check.
    /// This does NOT check all.
    /// Eg. does not check if binds for activity are not missing in some actions where WF needs them.
    pub fn assert_valid_proposal_binds_structure(
        &self,
        binds: &[Option<ActivityBind>],
        activities: &[Activity],
    ) {
        todo!();
        /*
         assert_eq!(
            binds.len(),
            activities.len() - 1,
            "Binds must be same length as activities."
        );
        // Skip init activity.
        for (idx, act) in activities.iter().skip(1).enumerate() {
            match act {
                Activity::Init => panic!("Invalid WF. Init activity defined at > 0 index."),
                Activity::DaoActivity(a) | Activity::FnCallActivity(a) => {
                    let act_binds = &binds[idx];

                    // Skip binds with activity which does not have filled
                    if act_binds.is_none() {
                        continue;
                    } else {
                        assert_eq!(
                            act_binds.as_ref().unwrap().values.len(),
                            a.actions.as_slice().len(),
                            "Activity action binds does not have same len."
                        );
                    }
                }
            }
        }
        */
    }

    /// Binds dao FnCall
    pub fn get_fncall_id_with_metadata(
        &self,
        id: FnCallIdType,
        sources: &dyn Source,
    ) -> Result<(AccountId, MethodName, Vec<ObjectMetadata>), ActionError> {
        let data = match id {
            FnCallIdType::Static((account, method)) => {
                if account.as_str() == "self" {
                    let name = env::current_account_id();
                    (
                        AccountId::try_from(name.to_string())
                            .map_err(|_| ActionError::InvalidDataType)?,
                        method.clone(),
                        self.function_call_metadata
                            .get(&(name.clone(), method.clone()))
                            .ok_or(ActionError::MissingFnCallMetadata(method))?,
                    )
                } else {
                    (
                        account.clone(),
                        method.clone(),
                        self.function_call_metadata
                            .get(&(account, method.clone()))
                            .ok_or(ActionError::MissingFnCallMetadata(method))?,
                    )
                }
            }
            FnCallIdType::Dynamic(arg_src, method) => {
                let name = get_value_from_source(sources, &arg_src)
                    .map_err(ProcessingError::Source)?
                    .try_into_string()?;
                (
                    AccountId::try_from(name.to_string())
                        .map_err(|_| ActionError::InvalidDataType)?,
                    method.clone(),
                    self.function_call_metadata
                        .get(&(
                            AccountId::try_from(name.to_string())
                                .map_err(|_| ActionError::InvalidDataType)?,
                            method.clone(),
                        ))
                        .ok_or(ActionError::MissingFnCallMetadata(method))?,
                )
            }
            FnCallIdType::StandardStatic((account, method)) => {
                if account.as_str() == "self" {
                    let name = env::current_account_id();
                    (
                        name.clone(),
                        method.clone(),
                        self.standard_function_call_metadata
                            .get(&method.clone())
                            .ok_or(ActionError::MissingFnCallMetadata(method))?,
                    )
                } else {
                    (
                        account.clone(),
                        method.clone(),
                        self.function_call_metadata
                            .get(&(account, method.clone()))
                            .ok_or(ActionError::MissingFnCallMetadata(method))?,
                    )
                }
            }
            FnCallIdType::StandardDynamic(arg_src, method) => {
                let name = get_value_from_source(sources, &arg_src)
                    .map_err(ProcessingError::Source)?
                    .try_into_string()?;
                (
                    AccountId::try_from(name.to_string())
                        .map_err(|_| ActionError::InvalidDataType)?,
                    method.clone(),
                    self.standard_function_call_metadata
                        .get(&name)
                        .ok_or(ActionError::MissingFnCallMetadata(method))?,
                )
            }
        };
        Ok(data)
    }

    /// Adds `event` to event queue at `tick`.
    /// Requires `tick` to be >= current_timestamp and exact tick timestamp.
    pub fn dispatch_tick_event(&mut self, event: Event, tick: TimestampSec) {
        let current_timestamp = current_timestamp_sec();
        require!(tick >= current_timestamp, "tick time cannot be in past");
        require!(
            tick % self.tick_interval == 0,
            "tick must be exact tick timestamp"
        );

        let mut queue = self.events.get(&tick).unwrap_or_else(EventQueue::new);
        queue.add_event(event);
        self.events.insert(&tick, &queue);
    }
}

pub mod utils {
    use library::functions::utils::{
        into_storage_key_wrapper_str, into_storage_key_wrapper_u16, StorageKeyWrapper,
    };
    use near_sdk::env;

    use crate::{
        constants::{GROUP_RELEASE_PREFIX, STORAGE_BUCKET_PREFIX},
        GroupId, TimestampSec,
    };

    pub fn get_group_key(id: GroupId) -> StorageKeyWrapper {
        into_storage_key_wrapper_u16(GROUP_RELEASE_PREFIX, id)
    }

    pub fn get_bucket_id(id: &str) -> StorageKeyWrapper {
        into_storage_key_wrapper_str(STORAGE_BUCKET_PREFIX, id)
    }

    pub fn current_timestamp_sec() -> TimestampSec {
        env::block_timestamp() / 10u64.pow(9)
    }
}

#[derive(Default)]
pub struct DaoConsts;

impl Consts for DaoConsts {
    fn get(&self, key: u8) -> Option<Value> {
        match key {
            C_DAO_ACC_ID => Some(Value::String(env::current_account_id().to_string())),
            _ => None,
        }
    }
}

/// Helper data struct during activity execution.
pub struct ActivityContext {
    pub caller: AccountId,
    pub proposal_id: u32,
    pub activity_id: usize,
    pub attached_deposit: Balance,
    pub proposal_settings: ProposeSettings,
    pub actions_done_before: u8,
    pub actions_done_now: u8,
    pub activity_postprocessing: Option<Postprocessing>,
    pub terminal: Terminality,
    pub actions: Vec<TemplateAction>,
    pub optional_actions: u8,
}

#[allow(clippy::too_many_arguments)]
impl ActivityContext {
    pub fn new(
        proposal_id: u32,
        activity_id: usize,
        caller: AccountId,
        attached_deposit: Balance,
        proposal_settings: ProposeSettings,
        actions_done: u8,
        activity_postprocessing: Option<Postprocessing>,
        terminal: Terminality,
        actions: Vec<TemplateAction>,
    ) -> Self {
        Self {
            caller,
            proposal_id,
            activity_id,
            attached_deposit,
            proposal_settings,
            actions_done_before: actions_done,
            actions_done_now: actions_done,
            activity_postprocessing,
            terminal,
            actions,
            optional_actions: 0,
        }
    }

    pub fn actions_done(&self) -> u8 {
        self.actions_done_now - self.actions_done_before
    }

    pub fn set_next_action_done(&mut self) {
        self.actions_done_now += 1;
    }

    pub fn set_next_optional_action_done(&mut self) {
        self.optional_actions += 1;
    }

    pub fn actions_count(&self) -> u8 {
        self.actions.len() as u8
    }

    pub fn activity_autofinish(&self) -> bool {
        self.terminal == Terminality::Automatic
    }

    pub fn all_actions_done(&self) -> bool {
        self.actions_done_now == self.actions_count()
    }
}
