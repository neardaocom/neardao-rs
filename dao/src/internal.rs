use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
};

use near_sdk::{
    env::{self},
    serde_json, AccountId, Balance, Promise,
};

use crate::{
    callbacks::ext_self,
    constants::{C_DAO_ACC_ID, GLOBAL_BUCKET_IDENT, TGAS},
    core::{ActivityLog, Contract},
    error::{
        ActionError, ActivityError, ERR_DISTRIBUTION_ACC_EMPTY, ERR_DISTRIBUTION_MIN_VALUE,
        ERR_DISTRIBUTION_NOT_ENOUGH_FT, ERR_GROUP_HAS_NO_LEADER, ERR_GROUP_NOT_FOUND,
        ERR_LOCK_AMOUNT_OVERFLOW, ERR_STORAGE_BUCKET_EXISTS,
    },
    group::{Group, GroupInput, GroupMember, GroupMembers, GroupSettings, GroupTokenLockInput},
    helper::{
        deserialize::{
            deserialize_dao_settings, deserialize_group_input, deserialize_group_members,
            deserialize_group_settings,
        },
        get_datatype, get_datatype_from_values,
    },
    proposal::{Proposal, ProposalState},
    settings::DaoSettings,
    tags::{TagInput, Tags},
    token_lock::{TokenLock, UnlockMethod, UnlockPeriod},
    CalculatedVoteResults, InstanceWf, ProposalId, ProposalWf, VoteTotalPossible, Votes,
};
use library::{
    functions::serialize_to_json,
    storage::StorageBucket,
    types::DataType,
    workflow::{
        activity::{
            ActionInput, Activity, DaoActionData, Postprocessing, TemplateAction, TemplateActivity,
            Transition,
        },
        expression::Expression,
        instance::{Instance, InstanceState},
        settings::{ActivityBind, TemplateSettings},
        template::Template,
        types::{
            ActivityResult, ActivityRight, DaoActionIdent, FnCallMetadata, ValueContainer,
            VoteScenario,
        },
    },
    Consts, EventCode, FnCallId, MethodName, ObjectValues, TransitionId,
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
        metadata: Vec<Vec<FnCallMetadata>>,
    ) {
        for (i, c) in calls.iter().enumerate() {
            self.function_call_metadata.insert(c, &metadata[i]);
        }
    }

    /// Version of `init_function_calls` method but for standard interfaces.
    pub fn init_standard_function_calls(
        &mut self,
        calls: Vec<MethodName>,
        metadata: Vec<Vec<FnCallMetadata>>,
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

    pub fn get_wf_and_proposal(&self, proposal_id: u32) -> ProposalWf {
        let proposal = Proposal::from(self.proposals.get(&proposal_id).expect("Unknown proposal"));
        let (wft, mut wfs) = self.workflow_template.get(&proposal.workflow_id).unwrap();
        let settings = wfs.swap_remove(proposal.workflow_settings_id as usize);

        (proposal, wft, settings)
    }

    // TODO unit tests
    pub fn check_rights(&self, rights: &[ActivityRight], account_id: &AccountId) -> bool {
        if rights.len() == 0 {
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
                    if name != account_id {
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
                ActivityRight::TokenHolder => match self.ft.accounts.get(account_id) {
                    Some(ft) if ft > 0 => {
                        return true;
                    }
                    _ => continue,
                },
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
                ActivityRight::Member => {
                    match self.ft.accounts.get(account_id) {
                        Some(ft) if ft > 0 => {
                            return true;
                        }
                        _ => {
                            // Yep this is expensive...
                            // Iterate all groups and all members
                            let groups = self.groups.to_vec();

                            match groups
                                .into_iter()
                                .any(|(_, g)| g.get_member_by_account(account_id).is_some())
                            {
                                true => return true,
                                false => continue,
                            }
                        }
                    }
                }
                ActivityRight::Account(a) => match a == account_id {
                    true => return true,
                    false => continue,
                },
            }
        }
        false
    }

    // TODO: Test coverage.
    /// Evaluates vote results by scenario and type of voters.
    /// Returns tuple (max_possible_amount,vote_results)
    pub fn calculate_votes(
        &self,
        votes: &HashMap<String, u8>,
        scenario: &VoteScenario,
        vote_target: &ActivityRight,
    ) -> CalculatedVoteResults {
        let mut vote_result: Votes = [0 as u128; 3];
        let mut max_possible_amount: VoteTotalPossible = 0;
        match scenario {
            VoteScenario::Democratic => {
                match vote_target {
                    ActivityRight::Anyone => {
                        max_possible_amount = votes.len() as u128;
                    }
                    ActivityRight::Group(g) => match self.groups.get(&g) {
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
                        max_possible_amount = self.ft.token_holders_count as u128;
                    }
                    // If member exists in 2 groups, then he is accounted twice.
                    ActivityRight::Member => {
                        max_possible_amount = self.total_members_count as u128;
                    }
                    ActivityRight::GroupRole(g, r) => match self.groups.get(&g) {
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
                ActivityRight::Anyone | ActivityRight::TokenHolder => {
                    //TODO refactor Member - wont be calculated correctly
                    max_possible_amount = self.ft_total_distributed as u128 * self.decimal_const;

                    for (voter, vote_value) in votes.iter() {
                        vote_result[*vote_value as usize] +=
                            self.ft.accounts.get(voter).unwrap_or(0);
                    }
                }
                // This is expensive scenario
                ActivityRight::Member => {
                    let mut map = HashMap::with_capacity(64);
                    for group in self.groups.values_as_vector().iter() {
                        let members = group.get_members_accounts();
                        for member in members.into_iter() {
                            let amount = self.ft.accounts.get(&member).unwrap_or(0);

                            // AccountId can be in multiple groups
                            if map.insert(member, amount).is_none() {
                                max_possible_amount += amount;
                            }
                        }
                    }

                    for (voter, vote_value) in votes.iter() {
                        vote_result[*vote_value as usize] += *map.get(voter).unwrap_or(&0);
                    }
                }
                ActivityRight::Group(gid) => {
                    let group = self.groups.get(&gid).unwrap();
                    let members: Vec<AccountId> = group.get_members_accounts();

                    // Store it in temp hashmap so we dont have to IO ft_amount for each member again
                    let mut map = HashMap::with_capacity(members.len());
                    for member in members.into_iter() {
                        let amount = self.ft.accounts.get(&member).unwrap_or(0);
                        map.insert(member, amount);
                        max_possible_amount += amount;
                    }

                    for (voter, vote_value) in votes.iter() {
                        vote_result[*vote_value as usize] += *map.get(voter).unwrap_or(&0);
                    }
                }
                ActivityRight::GroupRole(gid, rid) => {
                    let group = self.groups.get(&gid).unwrap();
                    let members: Vec<AccountId> = group.get_members_accounts_by_role(*rid);

                    let mut map = HashMap::with_capacity(members.len());
                    for member in members.into_iter() {
                        let amount = self.ft.accounts.get(&member).unwrap_or(0);
                        map.insert(member, amount);
                        max_possible_amount += amount;
                    }

                    for (voter, vote_value) in votes.iter() {
                        vote_result[*vote_value as usize] += *map.get(voter).unwrap_or(&0);
                    }
                }
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
    pub fn distribute_ft(&mut self, amount: u32, account_ids: &[AccountId]) {
        assert!(account_ids.len() > 0, "{}", ERR_DISTRIBUTION_ACC_EMPTY);
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
            if !self.ft.accounts.contains_key(&acc) {
                self.ft.accounts.insert(&acc, &0);
            }

            self.ft
                .internal_transfer(&contract_account_id, acc, amount_per_acc, None);
        }
    }

    /// Error callback.
    /// If promise did not have to succeed, then instance is still updated.
    pub fn postprocessing_failed(
        &mut self,
        proposal_id: u32,
        action_id: u8,
        must_succeed: bool,
    ) -> Result<(), ActionError> {
        let (mut wfi, settings) = self.workflow_instance.get(&proposal_id).unwrap();

        let awaiting_state = wfi.awaiting_state.take().unwrap();

        let result = if must_succeed {
            // Switch state back to running.
            wfi.unset_awaiting_state(InstanceState::Running);

            Err(ActionError::PromiseFailed(
                awaiting_state.activity_id,
                action_id,
            ))
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

            Ok(())
        };

        self.workflow_instance
            .insert(&proposal_id, &(wfi, settings));

        result
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
    ) -> Result<(), ActionError> {
        let (mut wfi, settings) = self.workflow_instance.get(&proposal_id).unwrap();

        let awaiting_state = wfi.awaiting_state.take().unwrap();

        // Action transaction check if previous action succesfully finished.
        if wfi.actions_done_count < action_id {
            wfi.unset_awaiting_state(InstanceState::Running);
            self.workflow_instance
                .insert(&proposal_id, &(wfi, settings));
            return Err(ActionError::PromiseFailed(
                awaiting_state.activity_id,
                action_id,
            ));
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
        let result = match postprocessing {
            Some(pp) => {
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
                        &mut storage.as_mut(),
                        &mut global_storage,
                        &mut new_template,
                    )
                    .is_err()
                {
                    wfi.unset_awaiting_state(InstanceState::FatalError);
                    Err(ActionError::ActionPostprocessing(action_id))
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
                    Ok(())
                }
            }
            _ => Ok(()),
        };

        wfi.awaiting_state = Some(awaiting_state);
        self.workflow_instance
            .insert(&proposal_id, &(wfi, settings));
        result
    }

    /// Closure which might be required in workflow.
    /// Returns DAO's specific values which cannot be known ahead of time.
    pub fn dao_consts(&self) -> Box<Consts> {
        Box::new(|id| match id {
            C_DAO_ACC_ID => Some(DataType::String(env::current_account_id())),
            _ => None,
        })
    }

    /// Action logging method.
    /// Will be moved to indexer when its ready.
    pub fn log_action(
        &mut self,
        proposal_id: ProposalId,
        caller: &str,
        action_id: u8,
        args: &[Vec<DataType>],
        args_collections: Option<&[Vec<DataType>]>,
    ) {
        let mut logs = self
            .workflow_activity_log
            .get(&proposal_id)
            .unwrap_or_else(|| Vec::with_capacity(1));

        logs.push(ActivityLog {
            caller: caller.to_string(),
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

    /// Checks if inputs structure is same as activity definition. Same order as activity's actions is required.
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
                    .expect("Missing action input."),
            ) {
                (_, Some(a)) => {
                    if !a.action.eq(&action.action_data) {
                        return false;
                    }
                }
                (false, None) => return false,
                _ => (),
            }
        }

        true
    }

    /// Executes DAO's native action.
    /// Inner methods panic when provided malformed inputs - structure/datatype.
    pub fn execute_dao_action(
        &mut self,
        proposal_id: u32,
        action_ident: DaoActionIdent,
        inputs: &mut Vec<Vec<DataType>>,
    ) -> Result<(), ActionError> {
        match action_ident {
            DaoActionIdent::GroupAdd => {
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
                self.group_remove_member(
                    get_datatype_from_values(inputs, 0, 0)?.try_into_u64()? as u16,
                    get_datatype_from_values(inputs, 0, 1)?.try_into_string()?,
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
                self.ft_distribute(group_id, amount, account_ids);
            }
            DaoActionIdent::TreasurySendFt => {
                let (ft_account_id, receiver_id, amount, memo, msg) = (
                    get_datatype_from_values(inputs, 0, 0)?.try_into_string()?,
                    get_datatype_from_values(inputs, 0, 1)?.try_into_string()?,
                    get_datatype_from_values(inputs, 0, 2)?.try_into_u128()?,
                    get_datatype_from_values(inputs, 0, 3)?
                        .try_into_string()
                        .ok(),
                    None,
                );
                self.treasury_send_ft(ft_account_id, receiver_id, amount, memo, msg, false);
            }
            DaoActionIdent::TreasurySendFtContract => {
                let (ft_account_id, receiver_id, amount, memo, msg) = (
                    get_datatype_from_values(inputs, 0, 0)?.try_into_string()?,
                    get_datatype_from_values(inputs, 0, 1)?.try_into_string()?,
                    get_datatype_from_values(inputs, 0, 2)?.try_into_u128()?,
                    get_datatype_from_values(inputs, 0, 3)?
                        .try_into_string()
                        .ok(),
                    get_datatype_from_values(inputs, 0, 4)?
                        .try_into_string()
                        .ok(),
                );
                self.treasury_send_ft(ft_account_id, receiver_id, amount, memo, msg, true);
            }
            DaoActionIdent::TreasurySendNft => {
                let (nft_account_id, receiver_id, nft_id, memo, approval_id, msg) = (
                    get_datatype_from_values(inputs, 0, 0)?.try_into_string()?,
                    get_datatype_from_values(inputs, 0, 1)?.try_into_string()?,
                    get_datatype_from_values(inputs, 0, 2)?.try_into_string()?,
                    get_datatype_from_values(inputs, 0, 3)?
                        .try_into_string()
                        .ok(),
                    get_datatype_from_values(inputs, 0, 4)?.try_into_u64()? as u32,
                    None,
                );

                self.treasury_send_nft(
                    nft_account_id,
                    receiver_id,
                    nft_id,
                    memo,
                    approval_id,
                    msg,
                    false,
                );
            }
            DaoActionIdent::TreasurySendNFtContract => {
                let (nft_account_id, receiver_id, nft_id, memo, approval_id, msg) = (
                    get_datatype_from_values(inputs, 0, 0)?.try_into_string()?,
                    get_datatype_from_values(inputs, 0, 1)?.try_into_string()?,
                    get_datatype_from_values(inputs, 0, 2)?.try_into_string()?,
                    get_datatype_from_values(inputs, 0, 3)?
                        .try_into_string()
                        .ok(),
                    get_datatype_from_values(inputs, 0, 4)?.try_into_u64()? as u32,
                    get_datatype_from_values(inputs, 0, 5)?
                        .try_into_string()
                        .ok(),
                );
                self.treasury_send_nft(
                    nft_account_id,
                    receiver_id,
                    nft_id,
                    memo,
                    approval_id,
                    msg,
                    true,
                );
            }
            DaoActionIdent::TreasurySendNear => {
                let (receiver, amount) = (
                    get_datatype_from_values(inputs, 0, 0)?.try_into_string()?,
                    get_datatype_from_values(inputs, 0, 1)?.try_into_u128()?,
                );
                self.treasury_send_near(receiver, amount);
            }
            _ => unreachable!(),
        }

        Ok(())
    }

    pub fn execute_fn_call_action(
        &mut self,
        mut receiver: String,
        method: String,
        inputs: &[Vec<DataType>],
        deposit: u128,
        tgas: u16,
        metadata: &[FnCallMetadata],
    ) -> Promise {
        if receiver == "self" {
            receiver = env::current_account_id();
        }

        let args = serialize_to_json(inputs, metadata, 0);

        Promise::new(receiver).function_call(
            method.into_bytes(),
            args.into_bytes(),
            deposit,
            tgas as u64 * TGAS,
        )
    }

    /// Proposal binds structure check.
    /// This does NOT check all.
    /// Eg. does not check if binds for activity are not missing in some actions where WF needs them.
    pub fn assert_valid_proposal_binds_structure(
        &self,
        binds: &[Option<ActivityBind>],
        activities: &[Activity],
    ) {
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
    }
}

pub mod utils {
    use crate::{
        append,
        constants::{GROUP_RELEASE_PREFIX, STORAGE_BUCKET_PREFIX},
        core::StorageKeyWrapper,
        GroupId,
    };

    pub fn get_group_key(id: GroupId) -> StorageKeyWrapper {
        append(GROUP_RELEASE_PREFIX, &id.to_le_bytes()).into()
    }

    pub fn get_bucket_id(id: &str) -> StorageKeyWrapper {
        append(STORAGE_BUCKET_PREFIX, id.as_bytes()).into()
    }
}
