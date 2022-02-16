use library::types::{ActionIdent, DataType};
use library::utils::{args_to_json, bind_args, validate_args};
use library::{
    workflow::{ActionResult, Instance, InstanceState, ProposeSettings, TemplateSettings},
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
        unimplemented!();

        /*         self.add_group(GroupInput {
            settings,
            members,
            release: token_lock,
        }); */
    }
    pub fn group_remove(&mut self, proposal_id: ProposalId, id: GroupId) {
        unimplemented!();

        /*         match self.groups.remove(&id) {
            Some(mut group) => {
                let release: ReleaseDb = group.remove_storage_data().data.into();
                self.ft_total_locked -= release.total - release.init_distribution;
                self.total_members_count -= group.members.members_count() as u32;
            }
            _ => (),
        } */
    }
    pub fn group_update(&mut self, proposal_id: ProposalId, id: GroupId, settings: GroupSettings) {
        unimplemented!();

        /*         match self.groups.get(&id) {
            Some(mut group) => {
                group.settings = settings;
                self.groups.insert(&id, &group);
            }
            _ => (),
        } */
    }
    pub fn group_add_members(
        &mut self,
        proposal_id: ProposalId,
        id: GroupId,
        members: Vec<GroupMember>,
    ) -> ActionResult {
        let result = self.execute_action(
            env::predecessor_account_id(),
            proposal_id,
            ActionIdent::GroupAddMembers,
            None,
            None,
            &mut vec![vec![DataType::U16(id)]],
            &mut vec![vec![]], //TODO format
            None,
        );

        if result == ActionResult::Ok || result == ActionResult::Postprocessing {
            match self.groups.get(&id) {
                Some(mut group) => {
                    self.total_members_count += group.add_members(members);
                    self.groups.insert(&id, &group);
                }
                _ => (),
            }
        }

        result
    }

    pub fn group_remove_member(&mut self, proposal_id: ProposalId, id: GroupId, member: AccountId) {
        unimplemented!();
        /*
        match self.groups.get(&id) {
            Some(mut group) => {
                group.remove_member(member);
                self.total_members_count -= 1;
                self.groups.insert(&id, &group);
            }
            _ => (),
        } */
    }
    pub fn settings_update(&mut self, proposal_id: ProposalId, settings: DaoSettings) {
        unimplemented!();
        /*         assert_valid_dao_settings(&settings);
        self.settings.replace(&settings.into()); */
    }

    pub fn media_add(&mut self, proposal_id: ProposalId, media: Media) -> ActionResult {
        let result = self.execute_action(
            env::predecessor_account_id(),
            proposal_id,
            ActionIdent::MediaAdd,
            None,
            None,
            &mut vec![vec![]],
            &mut vec![vec![]], //TODO format
            None,
        );

        if result == ActionResult::Ok || result == ActionResult::Postprocessing {
            self.media_last_id += 1;
            self.media.insert(&self.media_last_id, &media);
        }

        result
    }
    pub fn media_invalidate(&mut self, proposal_id: ProposalId, id: u32) {
        unimplemented!();

        /*         match self.media.get(&id) {
            Some(mut media) => {
                media.valid = false;
                self.media.insert(&id, &media);
            }
            _ => (),
        } */
    }
    pub fn media_remove(&mut self, proposal_id: ProposalId, id: u32) -> Option<Media> {
        unimplemented!();
        /* self.media.remove(&id) */
    }

    pub fn tag_add(
        &mut self,
        proposal_id: ProposalId,
        category: TagCategory,
        tags: Vec<String>,
    ) -> Option<(TagId, TagId)> {
        unimplemented!();
        /*
        let mut t = self.tags.get(&category).unwrap_or(Tags::new());
        let ids = t.insert(tags);
        self.tags.insert(&category, &t);
        ids */
    }

    pub fn tag_edit(
        &mut self,
        proposal_id: ProposalId,
        category: TagCategory,
        id: TagId,
        value: String,
    ) {
        unimplemented!();
        /*
        match self.tags.get(&category) {
            Some(mut t) => {
                t.rename(id, value);
                self.tags.insert(&category, &t);
            }
            None => (),
        } */
    }

    pub fn tag_remove(&mut self, proposal_id: ProposalId, category: TagCategory, id: TagId) {
        unimplemented!();
        /*
        match self.tags.get(&category) {
            Some(mut t) => {
                t.remove(id);
                self.tags.insert(&category, &t);
            }
            None => (),
        } */
    }

    //TODO add rights ??
    pub fn ft_unlock(&mut self, group_ids: Vec<GroupId>) -> Vec<u32> {
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
    ) -> ActionResult {
        let mut args = vec![vec![
            DataType::U16(group_id),
            DataType::U32(amount),
            DataType::VecString(account_ids),
        ]];

        let result = self.execute_action(
            env::predecessor_account_id(),
            proposal_id,
            ActionIdent::FtDistribute,
            None,
            None,
            &mut args,
            &mut vec![vec![]], //TODO format
            None,
        );

        if result == ActionResult::Ok || result == ActionResult::Postprocessing {
            if let Some(mut group) = self
                .groups
                .get(&(args[0][0].clone().try_into_u128().unwrap() as u16))
            //TODO improve
            {
                let account_ids = args[0].pop().unwrap().try_into_vec_str().unwrap();
                let amount = args[0].pop().unwrap().try_into_u128().unwrap() as u32;
                match group.distribute_ft(amount) && account_ids.len() > 0 {
                    true => {
                        self.groups.insert(&group_id, &group);
                        self.distribute_ft(amount, &account_ids);
                    }
                    _ => (),
                }
            }
        }

        result
    }

    pub fn treasury_send_near(
        &mut self,
        proposal_id: ProposalId,
        receiver_id: AccountId,
        amount: U128,
    ) -> ActionResult {
        let mut args = vec![vec![DataType::String(receiver_id), DataType::U128(amount)]];

        self.execute_action(
            env::predecessor_account_id(),
            proposal_id,
            ActionIdent::TreasurySendNear,
            None,
            None,
            &mut args,
            &mut vec![vec![]], //TODO format
            None,
        )
    }

    pub fn treasury_send_ft(
        &mut self,
        proposal_id: ProposalId,
        ft_account_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
    ) -> ActionResult {
        let mut args = vec![vec![
            DataType::String(ft_account_id),
            DataType::String(receiver_id),
            DataType::U128(amount),
            DataType::String(memo.unwrap_or_default()),
        ]];

        self.execute_action(
            env::predecessor_account_id(),
            proposal_id,
            ActionIdent::TreasurySendFt,
            None,
            None,
            &mut args,
            &mut vec![vec![]], //TODO format
            None,
        )
    }

    pub fn treasury_send_ft_contract(
        &mut self,
        proposal_id: ProposalId,
        ft_account_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> ActionResult {
        let mut args = vec![vec![
            DataType::String(ft_account_id),
            DataType::String(receiver_id),
            DataType::U128(amount),
            DataType::String(memo.unwrap_or_default()),
            DataType::String(msg),
        ]];

        self.execute_action(
            env::predecessor_account_id(),
            proposal_id,
            ActionIdent::TreasurySendFtContract,
            None,
            None,
            &mut args,
            &mut vec![vec![]], //TODO format
            None,
        )
    }

    //TODO check correct NFT usage
    pub fn treasury_send_nft(
        &mut self,
        proposal_id: ProposalId,
        nft_account_id: AccountId,
        receiver_id: String,
        nft_id: String,
        memo: Option<String>,
        approval_id: u32,
    ) -> ActionResult {
        let mut args = vec![vec![
            DataType::String(nft_account_id),
            DataType::String(receiver_id),
            DataType::String(nft_id),
            DataType::String(memo.unwrap_or_default()),
            DataType::U32(approval_id),
        ]];

        self.execute_action(
            env::predecessor_account_id(),
            proposal_id,
            ActionIdent::TreasurySendNft,
            None,
            None,
            &mut args,
            &mut vec![vec![]], //TODO format
            None,
        )
    }

    //TODO check correct NFT usage
    pub fn treasury_send_nft_contract(
        &mut self,
        proposal_id: ProposalId,
        nft_account_id: AccountId,
        receiver_id: String,
        nft_id: String,
        memo: Option<String>,
        approval_id: u32,
        msg: String,
    ) -> ActionResult {
        let mut args = vec![vec![
            DataType::String(nft_account_id),
            DataType::String(receiver_id),
            DataType::String(nft_id),
            DataType::String(memo.unwrap_or_default()),
            DataType::U32(approval_id),
            DataType::String(msg),
        ]];

        self.execute_action(
            env::predecessor_account_id(),
            proposal_id,
            ActionIdent::TreasurySendNFtContract,
            None,
            None,
            &mut args,
            &mut vec![vec![]], //TODO format
            None,
        )
    }

    /// Invokes custom function call
    /// FnCall arguments MUST have same datatypes as specified in its FnCallMetadata
    pub fn fn_call(
        &mut self,
        proposal_id: ProposalId,
        fncall_receiver: AccountId,
        fncall_method: MethodName,
        mut arg_values: Vec<Vec<DataType>>,
        mut arg_values_collection: Vec<Vec<DataType>>,
    ) -> ActionResult {
        self.execute_action(
            env::predecessor_account_id(),
            proposal_id,
            ActionIdent::FnCall,
            None,
            Some((fncall_receiver, fncall_method)),
            &mut arg_values,
            &mut arg_values_collection,
            None,
        )
    }

    pub fn workflow_add(&mut self, proposal_id: ProposalId, workflow_id: u16) -> ActionResult {
        self.execute_action(
            env::predecessor_account_id(),
            proposal_id,
            ActionIdent::WorkflowAdd,
            None,
            None,
            &mut vec![vec![DataType::U16(workflow_id)]],
            &mut vec![],
            None,
        )
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

    /// Represents custom event which does not invokes any action except transition in workflow and saving data to storage when needed
    #[payable]
    pub fn event(
        &mut self,
        proposal_id: ProposalId,
        code: String,
        mut args: Vec<DataType>,
    ) -> ActionResult {
        args.insert(0, DataType::String(env::predecessor_account_id()));

        self.execute_action(
            env::predecessor_account_id(),
            proposal_id,
            ActionIdent::Event,
            Some(code),
            None,
            &mut vec![args],
            &mut vec![],
            Some(env::attached_deposit()),
        )
    }
}
