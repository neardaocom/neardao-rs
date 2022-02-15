use library::types::{ActionIdent, DataType};
use library::utils::{args_to_json, bind_args, validate_args};
use library::{
    workflow::{ActivityResult, Instance, InstanceState, ProposeSettings, TemplateSettings},
    MethodName,
};

use near_sdk::json_types::U128;
use near_sdk::{env, near_bindgen, AccountId, Promise};
use near_sdk::{log, PromiseOrValue};

use crate::callbacks::ext_self;
use crate::constants::TGAS;
use crate::proposal::{Proposal, ProposalState, VProposal};
use crate::release::ReleaseDb;
use crate::settings::assert_valid_dao_settings;
use crate::settings::DaoSettings;
use crate::tags::Tags;
use crate::{calc_percent_u128_unchecked, TagCategory, TagId};
use crate::{
    core::*,
    group::{GroupInput, GroupMember, GroupReleaseInput, GroupSettings},
    media::Media,
    GroupId, ProposalId,
};

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn propose(
        &mut self,
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

        assert!(env::attached_deposit() >= settings.deposit_propose.unwrap_or(0.into()).0);

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
            env::block_timestamp() / 10u64.pow(9),
            caller,
            template_id,
            template_settings_id,
            true,
        );
        assert!(self.storage.get(&propose_settings.storage_key).is_none());

        self.proposals
            .insert(&self.proposal_last_id, &VProposal::Curr(proposal));
        self.workflow_instance.insert(
            &self.proposal_last_id,
            &(
                Instance::new(template_id, &wft.transitions),
                propose_settings,
            ),
        );

        self.proposal_last_id
    }

    #[payable]
    pub fn vote(&mut self, proposal_id: u32, vote_kind: u8) -> bool {
        if vote_kind > 2 {
            return false;
        }

        let caller = env::predecessor_account_id();
        let (mut proposal, _, wfs) = self.get_wf_and_proposal(proposal_id);

        assert!(env::attached_deposit() >= wfs.deposit_vote.unwrap_or(0.into()).0);

        if !self.check_rights(&[wfs.allowed_voters.clone()], &caller) {
            return false;
        }

        if proposal.state != ProposalState::InProgress
            || proposal.created + (wfs.duration as u64) < env::block_timestamp() / 10u64.pow(9)
        {
            //TODO update expired proposal state
            return false;
        }

        if wfs.vote_only_once && proposal.votes.contains_key(&caller) {
            return false;
        }

        proposal.votes.insert(caller, vote_kind);

        self.proposals
            .insert(&proposal_id, &VProposal::Curr(proposal));

        true
    }

    pub fn finish_proposal(&mut self, proposal_id: u32) -> ProposalState {
        let (mut proposal, wft, wfs) = self.get_wf_and_proposal(proposal_id);
        let (mut instance, mut settings) = self.workflow_instance.get(&proposal_id).unwrap();

        let new_state = match proposal.state {
            ProposalState::InProgress => {
                if proposal.created + wfs.duration as u64 > env::block_timestamp() / 10u64.pow(9) {
                    None
                } else {
                    // count votes
                    let (max_possible_amount, vote_results) =
                        self.calculate_votes(&proposal.votes, &wfs.scenario, &wfs.allowed_voters);
                    log!("{}, {:?}", max_possible_amount, vote_results);
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
                        // not enough quorum
                        Some(ProposalState::Invalid)
                    } else if calc_percent_u128_unchecked(
                        vote_results[1],
                        max_possible_amount,
                        self.decimal_const,
                    ) < wfs.approve_threshold
                    {
                        // not enough voters to accept
                        Some(ProposalState::Rejected)
                    } else {
                        // proposal is accepted, create new workflow activity with its storage
                        instance.state = InstanceState::Running;
                        // Storage key must be unique among all proposals
                        self.storage_bucket_add(settings.storage_key.as_str());
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
                state
            }
            None => proposal.state,
        }
    }

    pub fn group_add(
        &mut self,
        proposal_id: ProposalId,
        settings: GroupSettings,
        members: Vec<GroupMember>,
        token_lock: GroupReleaseInput,
    ) {
        let caller = env::predecessor_account_id();
        let (_, _, wfs) = self.get_wf_and_proposal(proposal_id);

        assert!(!self.check_rights(&wfs.allowed_proposers, &caller));

        self.add_group(GroupInput {
            settings,
            members,
            release: token_lock,
        });
    }
    pub fn group_remove(&mut self, proposal_id: ProposalId, id: GroupId) {
        let caller = env::predecessor_account_id();
        let (_, _, wfs) = self.get_wf_and_proposal(proposal_id);

        assert!(!self.check_rights(&wfs.allowed_proposers, &caller));

        match self.groups.remove(&id) {
            Some(mut group) => {
                let release: ReleaseDb = group.remove_storage_data().data.into();
                self.ft_total_locked -= release.total - release.init_distribution;
                self.total_members_count -= group.members.members_count() as u32;
            }
            _ => (),
        }
    }
    pub fn group_update(&mut self, proposal_id: ProposalId, id: GroupId, settings: GroupSettings) {
        let caller = env::predecessor_account_id();
        let (_, _, wfs) = self.get_wf_and_proposal(proposal_id);

        assert!(!self.check_rights(&wfs.allowed_proposers, &caller));

        match self.groups.get(&id) {
            Some(mut group) => {
                group.settings = settings;
                self.groups.insert(&id, &group);
            }
            _ => (),
        }
    }
    pub fn group_add_members(
        &mut self,
        proposal_id: ProposalId,
        id: GroupId,
        members: Vec<GroupMember>,
    ) -> PromiseOrValue<ActivityResult> {
        let caller = env::predecessor_account_id();
        let (proposal, wft, wfs) = self.get_wf_and_proposal(proposal_id);

        // proposal state check
        assert!(proposal.state == ProposalState::Accepted);

        let (mut wfi, settings) = self.workflow_instance.get(&proposal_id).unwrap();

        if wfi.state == InstanceState::Finished {
            return PromiseOrValue::Value(ActivityResult::Finished);
        }

        // transition check
        let (transition_id, activity_id): (u8, u8) = wfi
            .get_target_trans_with_act(&wft, ActionIdent::GroupAddMembers, None)
            .expect("Undefined transition");

        // rights checks
        assert!(self.check_rights(
            &wfs.activity_rights[activity_id as usize - 1].as_slice(),
            &caller
        ));

        let dao_consts = self.dao_consts();
        let mut bucket = self.storage.get(&settings.storage_key).unwrap();
        let mut args = vec![vec![DataType::U16(id)]];
        let mut args_collections = vec![vec![]];
        let (result, postprocessing) = wfi.transition_to_next(
            activity_id,
            transition_id,
            &wft,
            &dao_consts,
            &wfs,
            &settings,
            args.as_slice(),
            &mut bucket,
            0,
        );

        // arguments validation TODO
        if wft.obj_validators[activity_id as usize - 1].len() > 0
            && !validate_args(
                &dao_consts,
                &settings.binds,
                &wft.obj_validators[activity_id as usize - 1].as_slice(),
                &wft.validator_exprs.as_slice(),
                &bucket,
                &mut args.as_slice(),
                &mut args_collections.as_slice(),
                &[],
            )
        {
            return PromiseOrValue::Value(ActivityResult::ErrValidation);
        }

        let result = match result {
            ActivityResult::Ok => {
                self.log_action(
                    proposal_id,
                    caller.as_str(),
                    activity_id,
                    args.as_slice(),
                    None,
                );

                match self.groups.get(&id) {
                    Some(mut group) => {
                        self.total_members_count += group.add_members(members);
                        self.groups.insert(&id, &group);
                    }
                    _ => (),
                }

                let user_value = postprocessing
                    .as_ref()
                    .map(|p| p.try_to_get_inner_value(args.as_slice(), settings.binds.as_slice()))
                    .flatten();
                /*

                bind_args(
                    &dao_consts,
                    settings.binds.as_slice(),
                    wft.activities[activity_id as usize]
                        .as_ref()
                        .unwrap()
                        .activity_inputs
                        .as_slice(),
                    &mut bucket,
                    &mut args,
                    &mut vec![],
                    0,
                    0,
                ); */

                //TODO
                if postprocessing.is_some() {
                    PromiseOrValue::Promise(ext_self::postprocess(
                        proposal_id,
                        settings.storage_key.clone(),
                        postprocessing,
                        user_value,
                        &env::current_account_id(),
                        0,
                        30 * TGAS,
                    ))
                } else {
                    PromiseOrValue::Value(result)
                }
            }
            _ => PromiseOrValue::Value(result),
        };

        self.workflow_instance
            .insert(&proposal_id, &(wfi, settings))
            .unwrap();

        result
    }
    pub fn group_(&mut self, proposal_id: ProposalId, id: GroupId, member: AccountId) {
        let caller = env::predecessor_account_id();
        let (_, _, wfs) = self.get_wf_and_proposal(proposal_id);

        assert!(!self.check_rights(&wfs.allowed_proposers, &caller));

        match self.groups.get(&id) {
            Some(mut group) => {
                group.remove_member(member);
                self.total_members_count -= 1;
                self.groups.insert(&id, &group);
            }
            _ => (),
        }
    }
    pub fn settings_update(&mut self, proposal_id: ProposalId, settings: DaoSettings) {
        let caller = env::predecessor_account_id();
        let (_, _, wfs) = self.get_wf_and_proposal(proposal_id);

        assert!(!self.check_rights(&wfs.allowed_proposers, &caller));
        assert_valid_dao_settings(&settings);
        self.settings.replace(&settings.into());
    }
    //TODO
    pub fn media_add(
        &mut self,
        proposal_id: ProposalId,
        media: Media,
    ) -> PromiseOrValue<ActivityResult> {
        let caller = env::predecessor_account_id();
        let (proposal, wft, wfs) = self.get_wf_and_proposal(proposal_id);

        // proposal state check
        assert!(proposal.state == ProposalState::Accepted);

        let (mut wfi, settings) = self.workflow_instance.get(&proposal_id).unwrap();

        if wfi.state == InstanceState::Finished {
            return PromiseOrValue::Value(ActivityResult::Finished);
        }

        // transition check
        let (transition_id, activity_id): (u8, u8) = wfi
            .get_target_trans_with_act(&wft, ActionIdent::MediaAdd, None)
            .expect("Undefined transition");

        // rights checks
        assert!(self.check_rights(
            &wfs.activity_rights[activity_id as usize - 1].as_slice(),
            &caller
        ));

        let dao_consts = self.dao_consts();
        let mut bucket = self.storage.get(&settings.storage_key).unwrap();
        let mut args = vec![vec![]];
        let mut args_collections = vec![vec![]];
        let (result, postprocessing) = wfi.transition_to_next(
            activity_id,
            transition_id,
            &wft,
            &dao_consts,
            &wfs,
            &settings,
            args.as_slice(),
            &mut bucket,
            0,
        );

        // arguments validation TODO
        if wft.obj_validators[activity_id as usize - 1].len() > 0
            && !validate_args(
                &dao_consts,
                &settings.binds,
                &wft.obj_validators[activity_id as usize - 1].as_slice(),
                &wft.validator_exprs.as_slice(),
                &bucket,
                &mut args.as_slice(),
                &mut args_collections.as_slice(),
                &[],
            )
        {
            return PromiseOrValue::Value(ActivityResult::ErrValidation);
        }

        let result = match result {
            ActivityResult::Ok => {
                self.log_action(
                    proposal_id,
                    caller.as_str(),
                    activity_id,
                    args.as_slice(),
                    None,
                );

                self.media_last_id += 1;
                self.media.insert(&self.media_last_id, &media);

                let user_value = postprocessing
                    .as_ref()
                    .map(|p| p.try_to_get_inner_value(args.as_slice(), settings.binds.as_slice()))
                    .flatten();
                /*

                bind_args(
                    &dao_consts,
                    settings.binds.as_slice(),
                    wft.activities[activity_id as usize]
                        .as_ref()
                        .unwrap()
                        .activity_inputs
                        .as_slice(),
                    &mut bucket,
                    &mut args,
                    &mut vec![],
                    0,
                    0,
                ); */

                //TODO
                if postprocessing.is_some() {
                    PromiseOrValue::Promise(ext_self::postprocess(
                        proposal_id,
                        settings.storage_key.clone(),
                        postprocessing,
                        user_value,
                        &env::current_account_id(),
                        0,
                        30 * TGAS,
                    ))
                } else {
                    PromiseOrValue::Value(result)
                }
            }
            _ => PromiseOrValue::Value(result),
        };

        self.workflow_instance
            .insert(&proposal_id, &(wfi, settings))
            .unwrap();

        result
    }
    pub fn media_invalidate(&mut self, proposal_id: ProposalId, id: u32) {
        let caller = env::predecessor_account_id();
        let (_, _, wfs) = self.get_wf_and_proposal(proposal_id);

        assert!(!self.check_rights(&wfs.allowed_proposers, &caller));
        match self.media.get(&id) {
            Some(mut media) => {
                media.valid = false;
                self.media.insert(&id, &media);
            }
            _ => (),
        }
    }
    pub fn media_remove(&mut self, proposal_id: ProposalId, id: u32) -> Option<Media> {
        let caller = env::predecessor_account_id();
        let (_, _, wfs) = self.get_wf_and_proposal(proposal_id);

        assert!(!self.check_rights(&wfs.allowed_proposers, &caller));
        self.media.remove(&id)
    }

    pub fn tag_add(
        &mut self,
        proposal_id: ProposalId,
        category: TagCategory,
        tags: Vec<String>,
    ) -> Option<(TagId, TagId)> {
        let caller = env::predecessor_account_id();
        let (_, _, wfs) = self.get_wf_and_proposal(proposal_id);

        assert!(!self.check_rights(&wfs.allowed_proposers, &caller));
        let mut t = self.tags.get(&category).unwrap_or(Tags::new());
        let ids = t.insert(tags);
        self.tags.insert(&category, &t);
        ids
    }

    pub fn tag_edit(
        &mut self,
        proposal_id: ProposalId,
        category: TagCategory,
        id: TagId,
        value: String,
    ) {
        let caller = env::predecessor_account_id();
        let (_, _, wfs) = self.get_wf_and_proposal(proposal_id);

        assert!(!self.check_rights(&wfs.allowed_proposers, &caller));
        match self.tags.get(&category) {
            Some(mut t) => {
                t.rename(id, value);
                self.tags.insert(&category, &t);
            }
            None => (),
        }
    }

    pub fn tag_remove(&mut self, proposal_id: ProposalId, category: TagCategory, id: TagId) {
        let caller = env::predecessor_account_id();
        let (_, _, wfs) = self.get_wf_and_proposal(proposal_id);

        assert!(!self.check_rights(&wfs.allowed_proposers, &caller));
        //TODO implement check for all usage
        match self.tags.get(&category) {
            Some(mut t) => {
                t.remove(id);
                self.tags.insert(&category, &t);
            }
            None => (),
        }
    }

    //TODO add rights ??
    pub fn ft_unlock(&mut self, proposal_id: ProposalId, group_ids: Vec<GroupId>) -> Vec<u32> {
        let mut released = Vec::with_capacity(group_ids.len());
        for id in group_ids.into_iter() {
            if let Some(mut group) = self.groups.get(&id) {
                released.push(group.unlock_ft(env::block_timestamp()));
                self.groups.insert(&id, &group);
            }
        }
        released
    }
    pub fn ft_distribute(
        &mut self,
        proposal_id: ProposalId,
        group_id: u16,
        amount: u32,
        account_ids: Vec<AccountId>,
    ) -> PromiseOrValue<ActivityResult> {
        let caller = env::predecessor_account_id();
        let (proposal, wft, wfs) = self.get_wf_and_proposal(proposal_id);

        // proposal state check
        assert!(proposal.state == ProposalState::Accepted);

        let (mut wfi, settings) = self.workflow_instance.get(&proposal_id).unwrap();

        if wfi.state == InstanceState::Finished {
            return PromiseOrValue::Value(ActivityResult::Finished);
        }

        // transition check
        let (transition_id, activity_id): (u8, u8) = wfi
            .get_target_trans_with_act(&wft, ActionIdent::FtDistribute, None)
            .expect("Undefined transition");

        // rights checks
        assert!(self.check_rights(
            &wfs.activity_rights[activity_id as usize - 1].as_slice(),
            &caller
        ));

        let dao_consts = self.dao_consts();
        let mut bucket = self.storage.get(&settings.storage_key).unwrap();
        let mut args = vec![vec![]];
        let mut args_collections = vec![vec![]];
        let (result, postprocessing) = wfi.transition_to_next(
            activity_id,
            transition_id,
            &wft,
            &dao_consts,
            &wfs,
            &settings,
            args.as_slice(),
            &mut bucket,
            0,
        );

        // arguments validation TODO
        if wft.obj_validators[activity_id as usize - 1].len() > 0
            && !validate_args(
                &dao_consts,
                &settings.binds,
                &wft.obj_validators[activity_id as usize - 1].as_slice(),
                &wft.validator_exprs.as_slice(),
                &bucket,
                &mut args.as_slice(),
                &mut args_collections.as_slice(),
                &[],
            )
        {
            return PromiseOrValue::Value(ActivityResult::ErrValidation);
        }

        let result = match result {
            ActivityResult::Ok => {
                self.log_action(
                    proposal_id,
                    caller.as_str(),
                    activity_id,
                    args.as_slice(),
                    None,
                );

                if let Some(mut group) = self.groups.get(&group_id) {
                    match group.distribute_ft(amount) && account_ids.len() > 0 {
                        true => {
                            self.groups.insert(&group_id, &group);
                            self.distribute_ft(amount, &account_ids);
                        }
                        _ => (),
                    }
                }

                let user_value = postprocessing
                    .as_ref()
                    .map(|p| p.try_to_get_inner_value(args.as_slice(), settings.binds.as_slice()))
                    .flatten();
                /*

                bind_args(
                    &dao_consts,
                    settings.binds.as_slice(),
                    wft.activities[activity_id as usize]
                        .as_ref()
                        .unwrap()
                        .activity_inputs
                        .as_slice(),
                    &mut bucket,
                    &mut args,
                    &mut vec![],
                    0,
                    0,
                ); */

                //TODO
                if postprocessing.is_some() {
                    PromiseOrValue::Promise(ext_self::postprocess(
                        proposal_id,
                        settings.storage_key.clone(),
                        postprocessing,
                        user_value,
                        &env::current_account_id(),
                        0,
                        30 * TGAS,
                    ))
                } else {
                    PromiseOrValue::Value(result)
                }
            }
            _ => PromiseOrValue::Value(result),
        };

        self.workflow_instance
            .insert(&proposal_id, &(wfi, settings))
            .unwrap();

        result
    }
    pub fn treasury_send_near(
        &mut self,
        proposal_id: ProposalId,
        receiver_id: AccountId,
        amount: U128,
    ) -> PromiseOrValue<ActivityResult> {
        let caller = env::predecessor_account_id();
        let (proposal, wft, wfs) = self.get_wf_and_proposal(proposal_id);

        // proposal state check
        assert!(proposal.state == ProposalState::Accepted);

        let (mut wfi, settings) = self.workflow_instance.get(&proposal_id).unwrap();

        if wfi.state == InstanceState::Finished {
            return PromiseOrValue::Value(ActivityResult::Finished);
        }

        // transition check
        let (transition_id, activity_id): (u8, u8) = wfi
            .get_target_trans_with_act(&wft, ActionIdent::TreasurySendNear, None)
            .expect("Undefined transition");

        // rights checks
        assert!(self.check_rights(
            &wfs.activity_rights[activity_id as usize - 1].as_slice(),
            &caller
        ));

        let dao_consts = self.dao_consts();
        let mut bucket = self.storage.get(&settings.storage_key).unwrap();
        let mut args = vec![vec![DataType::String(receiver_id), DataType::U128(amount)]];
        let (result, postprocessing) = wfi.transition_to_next(
            activity_id,
            transition_id,
            &wft,
            &dao_consts,
            &wfs,
            &settings,
            args.as_slice(),
            &mut bucket,
            0,
        );

        // arguments validation
        if wft.obj_validators[activity_id as usize - 1].len() > 0
            && !validate_args(
                &dao_consts,
                &settings.binds,
                &wft.obj_validators[activity_id as usize - 1].as_slice(),
                &wft.validator_exprs.as_slice(),
                &bucket,
                &mut args.as_slice(),
                &mut vec![],
                &[],
            )
        {
            return PromiseOrValue::Value(ActivityResult::ErrValidation);
        }

        let result = match result {
            ActivityResult::Ok => {
                self.log_action(
                    proposal_id,
                    caller.as_str(),
                    activity_id,
                    args.as_slice(),
                    None,
                );

                bind_args(
                    &dao_consts,
                    settings.binds.as_slice(),
                    wft.activities[activity_id as usize]
                        .as_ref()
                        .unwrap()
                        .activity_inputs
                        .as_slice(),
                    &mut bucket,
                    &mut args,
                    &mut vec![],
                    0,
                    0,
                );

                let user_value = postprocessing
                    .as_ref()
                    .map(|p| p.try_to_get_inner_value(args.as_slice(), settings.binds.as_slice()))
                    .flatten();

                PromiseOrValue::Promise(
                    Promise::new(args[0].swap_remove(0).try_into_string().unwrap())
                        .transfer(args[0].swap_remove(0).try_into_u128().unwrap())
                        .then(ext_self::postprocess(
                            proposal_id,
                            settings.storage_key.clone(),
                            postprocessing,
                            user_value,
                            &env::current_account_id(),
                            0,
                            30 * TGAS,
                        )),
                )
            }
            _ => PromiseOrValue::Value(result),
        };

        self.workflow_instance
            .insert(&proposal_id, &(wfi, settings))
            .unwrap();

        result
    }

    pub fn treasury_send_ft(
        &mut self,
        proposal_id: ProposalId,
        ft_account_id: AccountId,
        receiver_id: AccountId,
        is_contract: bool,
        amount_ft: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<ActivityResult> {
        let caller = env::predecessor_account_id();
        let (proposal, wft, wfs) = self.get_wf_and_proposal(proposal_id);

        // proposal state check
        assert!(proposal.state == ProposalState::Accepted);

        let (mut wfi, settings) = self.workflow_instance.get(&proposal_id).unwrap();

        if wfi.state == InstanceState::Finished {
            return PromiseOrValue::Value(ActivityResult::Finished);
        }

        // transition check
        let (transition_id, activity_id): (u8, u8) = wfi
            .get_target_trans_with_act(&wft, ActionIdent::TreasurySendFt, None)
            .expect("Undefined transition");

        // rights checks
        assert!(self.check_rights(
            &wfs.activity_rights[activity_id as usize - 1].as_slice(),
            &caller
        ));

        let dao_consts = self.dao_consts();
        let mut bucket = self.storage.get(&settings.storage_key).unwrap();
        let mut args = vec![vec![]];
        let mut args_collections = vec![vec![]];
        let (result, postprocessing) = wfi.transition_to_next(
            activity_id,
            transition_id,
            &wft,
            &dao_consts,
            &wfs,
            &settings,
            args.as_slice(),
            &mut bucket,
            0,
        );

        // arguments validation TODO
        if wft.obj_validators[activity_id as usize - 1].len() > 0
            && !validate_args(
                &dao_consts,
                &settings.binds,
                &wft.obj_validators[activity_id as usize - 1].as_slice(),
                &wft.validator_exprs.as_slice(),
                &bucket,
                &mut args.as_slice(),
                &mut args_collections.as_slice(),
                &[],
            )
        {
            return PromiseOrValue::Value(ActivityResult::ErrValidation);
        }

        let result = match result {
            ActivityResult::Ok => {
                self.log_action(
                    proposal_id,
                    caller.as_str(),
                    activity_id,
                    args.as_slice(),
                    None,
                );

                let user_value = postprocessing
                    .as_ref()
                    .map(|p| p.try_to_get_inner_value(args.as_slice(), settings.binds.as_slice()))
                    .flatten();
                /*

                bind_args(
                    &dao_consts,
                    settings.binds.as_slice(),
                    wft.activities[activity_id as usize]
                        .as_ref()
                        .unwrap()
                        .activity_inputs
                        .as_slice(),
                    &mut bucket,
                    &mut args,
                    &mut vec![],
                    0,
                    0,
                ); */

                let mut promise = Promise::new(ft_account_id);
                if is_contract {
                    //TODO test formating memo
                    promise = promise.function_call(
                        b"ft_transfer_call".to_vec(),
                        format!(
                            "{{\"receiver_id\":\"{}\",\"amount\":\"{}\",\"memo\":\"{}\",\"msg\":\"{}\"}}",
                            receiver_id,
                            amount_ft.0,
                            memo.unwrap_or("".into()),
                            msg
                        )
                        .as_bytes()
                        .to_vec(),
                        0,
                        TGAS,
                    );
                } else {
                    promise = promise.function_call(
                        b"ft_transfer".to_vec(),
                        format!(
                            "{{\"receiver_id\":{},\"amount\":\"{}\",\"msg\":\"{}\"}}",
                            receiver_id, amount_ft.0, msg
                        )
                        .as_bytes()
                        .to_vec(),
                        0,
                        TGAS,
                    );
                }

                //TODO
                if postprocessing.is_some() {
                    PromiseOrValue::Promise(promise.then(ext_self::postprocess(
                        proposal_id,
                        settings.storage_key.clone(),
                        postprocessing,
                        user_value,
                        &env::current_account_id(),
                        0,
                        30 * TGAS,
                    )))
                } else {
                    PromiseOrValue::Promise(promise)
                }
            }
            _ => PromiseOrValue::Value(result),
        };

        self.workflow_instance
            .insert(&proposal_id, &(wfi, settings))
            .unwrap();

        result
    }
    //TODO check correct NFT usage
    pub fn treasury_send_nft(
        &mut self,
        proposal_id: ProposalId,
        nft_account_id: AccountId,
        nft_id: String,
        approval_id: String,
        receiver_id: String,
        is_contract: bool,
        memo: Option<String>,
        msg: String,
    ) -> Promise {
        unimplemented!();

        let caller = env::predecessor_account_id();
        let (_, _, wfs) = self.get_wf_and_proposal(proposal_id);

        assert!(!self.check_rights(&wfs.allowed_proposers, &caller));
        let promise = Promise::new(nft_account_id);
        if is_contract {
            //TODO test formating memo
            promise.function_call(b"nft_transfer_call".to_vec(), format!("{{\"receiver_id\":\"{}\",\"token_id\":\"{}\",\"approval_id\":{},\"memo\":\"{}\",\"msg\":\"{}\"}}", receiver_id, nft_id, approval_id, memo.unwrap_or("".into()), msg).as_bytes().to_vec(), 0, TGAS)
        } else {
            promise.function_call(
                b"nft_transfer".to_vec(),
                format!(
                    "{{\"receiver_id\":{},\"token_id\":\"{}\",\"approval_id\":{},\"memo\":\"{}\"}}",
                    receiver_id,
                    nft_id,
                    approval_id,
                    memo.unwrap_or("".into())
                )
                .as_bytes()
                .to_vec(),
                0,
                TGAS,
            )
        }
    }

    //TODO move to internal when properly tested
    /*     pub fn storage_add_bucket(&mut self, bucket_id: String) {
        self.storage_bucket_add(&bucket_id);
    }
    pub fn storage_remove_bucket(&mut self, bucket_id: String) {
        match self.storage.remove(&bucket_id) {
            Some(mut bucket) => {
                bucket.remove_storage_data();
            }
            None => (),
        }
    }

    pub fn storage_add_data(&mut self, bucket_id: String, data_id: String, data: DataType) {
        match self.storage.get(&bucket_id) {
            Some(mut bucket) => {
                bucket.add_data(&data_id, &data);
                self.storage.insert(&bucket_id, &bucket);
            }
            None => (),
        }
    }

    pub fn storage_remove_data(&mut self, bucket_id: String, data_id: String) -> Option<DataType> {
        match self.storage.get(&bucket_id) {
            Some(mut bucket) => {
                if let Some(data) = bucket.remove_data(&data_id) {
                    self.storage.insert(&bucket_id, &bucket);
                    Some(data)
                } else {
                    None
                }
            }
            None => None,
        }
    } */

    /// Invokes custom function call
    /// FnCall arguments MUST have same datatypes as specified in its FnCallMetadata
    pub fn fn_call(
        &mut self,
        proposal_id: ProposalId,
        fncall_receiver: AccountId,
        fncall_method: MethodName,
        mut arg_values: Vec<Vec<DataType>>,
        mut arg_values_collection: Vec<Vec<DataType>>,
    ) -> PromiseOrValue<ActivityResult> {
        let caller = env::predecessor_account_id();
        let (proposal, wft, wfs) = self.get_wf_and_proposal(proposal_id);

        assert!(proposal.state == ProposalState::Accepted);

        let (mut wfi, settings) = self.workflow_instance.get(&proposal_id).unwrap();

        if wfi.state == InstanceState::Finished {
            return PromiseOrValue::Value(ActivityResult::Finished);
        }

        //transition check
        let (transition_id, activity_id): (u8, u8) = wfi
            .get_target_trans_with_act(
                &wft,
                ActionIdent::FnCall,
                Some((fncall_receiver.clone(), fncall_method.clone())),
            )
            .expect("Undefined transition");

        log!("aid: {}, tid: {}", activity_id, transition_id);

        //rights checks
        assert!(self.check_rights(
            &wfs.activity_rights[activity_id as usize - 1].as_slice(),
            &caller
        ));

        let mut bucket = self.storage.get(&settings.storage_key).unwrap();
        let activity = wft.activities[activity_id as usize].as_ref().unwrap();

        log!("tgas: {}, deposit: {}", activity.tgas, activity.deposit.0);

        let receiver = if fncall_receiver.as_str() == "self" {
            env::current_account_id()
        } else {
            fncall_receiver.clone()
        };

        let dao_consts = self.dao_consts();

        // Everything should be provided by provider in correct format so unwraping is ok
        let fn_metadata = self
            .function_call_metadata
            .get(&(fncall_receiver, fncall_method.clone()))
            .unwrap();

        let (result, postprocessing) = wfi.transition_to_next(
            activity_id,
            transition_id,
            &wft,
            &dao_consts,
            &wfs,
            &settings,
            arg_values.as_slice(),
            &mut bucket,
            0,
        );

        if wft.obj_validators[activity_id as usize - 1].len() > 0
            && !validate_args(
                &dao_consts,
                &settings.binds,
                &wft.obj_validators[activity_id as usize - 1].as_slice(),
                &wft.validator_exprs.as_slice(),
                &bucket,
                arg_values.as_slice(),
                arg_values_collection.as_slice(),
                fn_metadata.as_slice(),
            )
        {
            return PromiseOrValue::Value(ActivityResult::ErrValidation);
        }

        let result = match result {
            ActivityResult::Ok => {
                let inner_value = postprocessing
                    .as_ref()
                    .map(|p| {
                        p.try_to_get_inner_value(arg_values.as_slice(), settings.binds.as_slice())
                    })
                    .flatten();

                log!("inner_value: {:#?}", inner_value);

                // bind args
                bind_args(
                    &dao_consts,
                    settings.binds.as_slice(),
                    wft.activities[activity_id as usize]
                        .as_ref()
                        .unwrap()
                        .activity_inputs
                        .as_slice(),
                    &mut bucket,
                    &mut arg_values,
                    &mut arg_values_collection,
                    0,
                    0,
                );

                //parse json object
                let args = args_to_json(
                    arg_values.as_slice(),
                    arg_values_collection.as_slice(),
                    &fn_metadata,
                    0,
                );

                PromiseOrValue::Promise(
                    Promise::new(receiver)
                        .function_call(
                            fncall_method.into_bytes(),
                            args.into_bytes(),
                            activity.deposit.0,
                            (activity.tgas as u64 * 10u64.pow(12)) as u64,
                        )
                        .then(ext_self::postprocess(
                            proposal_id,
                            settings.storage_key.clone(),
                            postprocessing,
                            inner_value,
                            &env::current_account_id(),
                            0,
                            30 * TGAS,
                        )),
                )
            }
            _ => PromiseOrValue::Value(result),
        };

        self.workflow_instance
            .insert(&proposal_id, &(wfi, settings))
            .unwrap();

        result
    }

    pub fn workflow_install(&mut self, proposal_id: ProposalId) {
        let caller = env::predecessor_account_id();
        let (_, _, wfs) = self.get_wf_and_proposal(proposal_id);

        assert!(!self.check_rights(&wfs.allowed_proposers, &caller));
        todo!()
    }

    pub fn workflow_add(
        &mut self,
        proposal_id: ProposalId,
        workflow_id: u16,
    ) -> PromiseOrValue<ActivityResult> {
        let caller = env::predecessor_account_id();
        let (proposal, wft, wfs) = self.get_wf_and_proposal(proposal_id);

        assert!(proposal.state == ProposalState::Accepted);

        let (mut wfi, settings) = self.workflow_instance.get(&proposal_id).unwrap();

        if wfi.state == InstanceState::Finished {
            return PromiseOrValue::Value(ActivityResult::Finished);
        }

        //transition check
        let (transition_id, activity_id): (u8, u8) = wfi
            .get_target_trans_with_act(&wft, ActionIdent::WorkflowAdd, None)
            .expect("Undefined transition");

        //rights checks
        assert!(self.check_rights(
            &wfs.activity_rights[activity_id as usize - 1].as_slice(),
            &caller
        ));

        let dao_consts = self.dao_consts();
        let mut bucket = self.storage.get(&settings.storage_key).unwrap();
        let mut args = vec![vec![DataType::U16(workflow_id)]];
        let (result, postprocessing) = wfi.transition_to_next(
            activity_id,
            transition_id,
            &wft,
            &dao_consts,
            &wfs,
            &settings,
            args.as_slice(),
            &mut bucket,
            0,
        );

        if wft.obj_validators[activity_id as usize - 1].len() > 0
            && !validate_args(
                &dao_consts,
                &settings.binds,
                &wft.obj_validators[activity_id as usize - 1].as_slice(),
                &wft.validator_exprs.as_slice(),
                &bucket,
                &mut args.as_slice(),
                &mut vec![],
                &[],
            )
        {
            return PromiseOrValue::Value(ActivityResult::ErrValidation);
        }

        let dao_settings: DaoSettings = self.settings.get().unwrap().into();
        let acc = env::current_account_id();
        let workflow_settings = self.proposed_workflow_settings.get(&proposal_id).unwrap();
        let result = match result {
            ActivityResult::Ok => {
                self.log_action(
                    proposal_id,
                    caller.as_str(),
                    activity_id,
                    args.as_slice(),
                    None,
                );

                // Not needed ATM
                //bind_args(
                //    &dao_consts,
                //    settings.binds.as_slice(),
                //    wft.activities[activity_id as usize]
                //        .as_ref()
                //        .unwrap()
                //        .activity_inputs
                //        .as_slice(),
                //    &mut bucket,
                //    &mut args,
                //    &mut vec![],
                //    0,
                //    0,
                //);

                PromiseOrValue::Promise(
                    Promise::new(dao_settings.workflow_provider)
                        .function_call(
                            b"wf_template".to_vec(),
                            format!(
                                "{{\"id\":{}}}",
                                args[0].pop().unwrap().try_into_u128().unwrap()
                            )
                            .into_bytes(),
                            0,
                            50 * TGAS,
                        )
                        .then(ext_self::store_workflow(
                            proposal_id,
                            workflow_settings,
                            &acc,
                            0,
                            50 * TGAS,
                        ))
                        .then(ext_self::postprocess(
                            proposal_id,
                            settings.storage_key.clone(),
                            postprocessing,
                            None,
                            &acc,
                            0,
                            30 * TGAS,
                        )),
                )
            }
            _ => PromiseOrValue::Value(result),
        };

        self.workflow_instance
            .insert(&proposal_id, &(wfi, settings))
            .unwrap();

        result
    }

    //TODO resolve other state combinations eg. FatalError on instance
    /// Changes workflow instance state to finish
    /// Rights to close are same as the "end" activity rights
    pub fn wf_finish(&mut self, proposal_id: u32) -> bool {
        let caller = env::predecessor_account_id();
        let (proposal, wft, wfs) = self.get_wf_and_proposal(proposal_id);

        assert!(proposal.state == ProposalState::Accepted);

        let (mut wfi, settings) = self.workflow_instance.get(&proposal_id).unwrap();

        if wfi.state == InstanceState::FatalError
            || self.check_rights(
                &wfs.activity_rights[wfi.current_activity_id as usize - 1].as_slice(),
                &caller,
            )
        {
            let result = wfi.finish(&wft);
            self.workflow_instance
                .insert(&proposal_id, &(wfi, settings));

            result
        } else {
            false
        }
    }
}
