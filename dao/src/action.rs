use library::types::{ActionIdent, DataType, DataTypeDef};
use library::utils::{bind_args, validate_args};
use library::{
    workflow::{
        ActivityResult, Instance, InstanceState, ProposeSettings, TemplateActivity,
        TemplateSettings,
    },
    MethodName,
};

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{Base64VecU8, WrappedDuration, WrappedTimestamp, U128, U64};
use near_sdk::serde::{self, Deserialize, Serialize};
use near_sdk::serde_json::{self, Value};
use near_sdk::{env, near_bindgen, AccountId, Promise};
use near_sdk::{log, PromiseOrValue};

use crate::callbacks::ext_self;
use crate::constants::TGAS;
use crate::errors::{
    ERR_GROUP_NOT_FOUND, ERR_NO_ACCESS, ERR_STORAGE_BUCKET_EXISTS, ERR_UNKNOWN_FNCALL,
};
use crate::group::Group;
use crate::internal::utils;
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
    GroupId, ProposalId
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

        assert!(env::attached_deposit() >= settings.deposit_propose.unwrap_or(0));

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
            env::block_timestamp(),
            caller,
            template_id,
            template_settings_id,
            true,
        );
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

        assert!(env::attached_deposit() >= wfs.deposit_vote.unwrap_or(0));

        if !self.check_rights(&[wfs.allowed_voters.clone()], &caller) {
            return false;
        }

        if proposal.state != ProposalState::InProgress
            || proposal.created + (wfs.duration) as u64 * 10u64.pow(9) < env::block_timestamp()
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
                if proposal.created + (wfs.duration) as u64 * 1000 > env::block_timestamp() {
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

    pub fn group_create(
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
    ) {
        let caller = env::predecessor_account_id();
        let (_, _, wfs) = self.get_wf_and_proposal(proposal_id);

        assert!(!self.check_rights(&wfs.allowed_proposers, &caller));

        match self.groups.get(&id) {
            Some(mut group) => {
                self.total_members_count += group.add_members(members);
                self.groups.insert(&id, &group);
            }
            _ => (),
        }
    }
    pub fn group_remove_member(&mut self, proposal_id: ProposalId, id: GroupId, member: AccountId) {
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
    pub fn media_add(&mut self, proposal_id: ProposalId, media: Media) {
        let caller = env::predecessor_account_id();
        let (_, _, wfs) = self.get_wf_and_proposal(proposal_id);

        assert!(!self.check_rights(&wfs.allowed_proposers, &caller));

        self.media_last_id += 1;
        self.media.insert(&self.media_last_id, &media);
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

    pub fn tag_insert(
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
        name: String,
    ) {
        let caller = env::predecessor_account_id();
        let (_, _, wfs) = self.get_wf_and_proposal(proposal_id);

        assert!(!self.check_rights(&wfs.allowed_proposers, &caller));
        match self.tags.get(&category) {
            Some(mut t) => {
                t.rename(id, name);
                self.tags.insert(&category, &t);
            }
            None => (),
        }
    }

    pub fn tag_clear(&mut self, proposal_id: ProposalId, category: TagCategory, id: TagId) {
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
    ) -> bool {
        let caller = env::predecessor_account_id();
        let (_, _, wfs) = self.get_wf_and_proposal(proposal_id);

        assert!(!self.check_rights(&wfs.allowed_proposers, &caller));
        if let Some(mut group) = self.groups.get(&group_id) {
            match group.distribute_ft(amount) && account_ids.len() > 0 {
                true => {
                    self.groups.insert(&group_id, &group);
                    self.distribute_ft(amount, &account_ids);
                    true
                }
                false => false,
            }
        } else {
            false
        }
    }
    pub fn treasury_near_send(
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

        // transition check
        let (transition_id, activity_id): (u8, u8) = wfi
            .get_target_trans_with_act(&wft, ActionIdent::NearSend)
            .expect("Undefined transition");

        // rights checks
        assert!(self.check_rights(
            &wfs.activity_rights[activity_id as usize - 1].as_slice(),
            &caller
        ));

        let mut bucket = self.storage.get(&settings.storage_key).unwrap();
        let mut args = vec![vec![
            DataType::String(receiver_id),
            DataType::U128(amount),
        ]];
        let (result, postprocessing) = wfi.transition_to_next(
            activity_id,
            transition_id,
            &wft,
            &settings,
            args.as_slice(),
            &mut bucket,
            0,
        );

        // arguments validation
        if settings.obj_validators[activity_id as usize - 1].len() > 0
            && !validate_args(
                &settings.binds,
                &settings.obj_validators[activity_id as usize - 1].as_slice(),
                &settings.validator_exprs.as_slice(),
                &bucket,
                &mut args.as_slice(),
                &mut vec![],
                &[],
            )
        {
            return PromiseOrValue::Value(ActivityResult::ErrValidation);
        }

        let dao_consts = self.dao_consts();
        let result = match result {
            ActivityResult::Ok => {
                // bind args
                bind_args(
                    &dao_consts,
                    settings.binds.as_slice(),
                    settings.activity_inputs[activity_id as usize - 1].as_slice(),
                    &mut bucket,
                    &mut args,
                    &mut vec![],
                    0,
                    0,
                );

                PromiseOrValue::Promise(
                    Promise::new(args[0].swap_remove(0).try_into_string().unwrap())
                        .transfer(args[0].swap_remove(0).try_into_u128().unwrap())
                        .then(ext_self::postprocess(
                            proposal_id,
                            settings.storage_key.clone(),
                            postprocessing,
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

    pub fn treasury_ft_send(
        &mut self,
        proposal_id: ProposalId,
        ft_account_id: AccountId,
        receiver_id: AccountId,
        is_contract: bool,
        amount_ft: U128,
        memo: Option<String>,
        msg: String,
    ) -> Promise {
        let caller = env::predecessor_account_id();
        let (_, _, wfs) = self.get_wf_and_proposal(proposal_id);

        assert!(!self.check_rights(&wfs.allowed_proposers, &caller));

        let promise = Promise::new(ft_account_id);
        if is_contract {
            //TODO test formating memo
            promise.function_call(
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
            )
        } else {
            promise.function_call(
                b"ft_transfer".to_vec(),
                format!(
                    "{{\"receiver_id\":{},\"amount\":\"{}\",\"msg\":\"{}\"}}",
                    receiver_id, amount_ft.0, msg
                )
                .as_bytes()
                .to_vec(),
                0,
                TGAS,
            )
        }
    }
    //TODO check correct NFT usage
    pub fn treasury_nft_send(
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
    pub fn storage_add_bucket(&mut self, bucket_id: String) {
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
    }

    /// Invokes registered function call
    pub fn function_call(
        &mut self,
        proposal_id: ProposalId,
        //fncall_receiver: AccountId,
        //fncall_method: MethodName,
        arg_names: Vec<Vec<String>>,
        mut arg_values: Vec<Vec<DataType>>,
        deposit: u128,
        tgas: u16,
    ) {
        let caller = env::predecessor_account_id();
        let (proposal, wft, wfs) = self.get_wf_and_proposal(proposal_id);

        assert!(proposal.state == ProposalState::Accepted);

        let (mut wfi, settings) = self.workflow_instance.get(&proposal_id).unwrap();

        //transition check
        let (transition_id, activity_id): (u8, u8) = wfi
            .get_target_trans_with_act(&wft, ActionIdent::FnCall)
            .expect("Undefined transition");

        //rights checks
        assert!(self.check_rights(
            &wfs.activity_rights[activity_id as usize - 1].as_slice(),
            &caller
        ));

        let mut bucket = self.storage.get(&settings.storage_key).unwrap();

        // Everything should be provided by provider so unwraping is ok
        let fncall_metadata = self.function_call_metadata.get(
            &wft.activities[activity_id as usize]
                .as_ref()
                .unwrap()
                .fncall_id
                .as_ref()
                .unwrap(),
        );

        // map metadata to check
        //bind_args(&settings, &mut arg_values, &bucket);
        /*
        let mut args = vec![];
        let (result, postprocessing) = wfi.transition_to_next(
            activity_id,
            transition_id,
            &wft,
            &settings,
            args.as_slice(),
            &mut bucket,
        ); */

        // TODO get constrains and binds from workflow template and postprocessing
        // Should be some match for Option

        // TODO validate fn args
        //fncall.bind_args()

        //add postprocessing (save promise result - must be from workflow)
    }

    pub fn function_call_add(
        &mut self,
        proposal_id: ProposalId,
        receiver_id: AccountId,
        method_name: MethodName,
    ) {
        /*         let caller = env::predecessor_account_id();
        let (_, _, wfs) = self.get_wf_and_proposal(proposal_id);

        assert!(!self.check_rights(&wfs.allowed_proposers, &caller));
        let id = format!("{}_{}", func.receiver, func.name);
        self.function_calls.insert(&id, &func); */
    }
    //TODO key as ID or func name
    pub fn function_call_remove(&mut self, proposal_id: ProposalId, id: String) {
        /*         let caller = env::predecessor_account_id();
        let (_, _, wfs) = self.get_wf_and_proposal(proposal_id);

        assert!(!self.check_rights(&wfs.allowed_proposers, &caller));
        self.function_calls.remove(&id); */
    }

    pub fn workflow_install(&mut self, proposal_id: ProposalId) {
        let caller = env::predecessor_account_id();
        let (_, _, wfs) = self.get_wf_and_proposal(proposal_id);

        assert!(!self.check_rights(&wfs.allowed_proposers, &caller));
        todo!()
    }

    // It makes no sense to check for something else than right to call this action in this case
    pub fn workflow_add(
        &mut self,
        proposal_id: ProposalId,
        workflow_id: u16,
    ) -> PromiseOrValue<ActivityResult> {
        let caller = env::predecessor_account_id();
        let (proposal, wft, wfs) = self.get_wf_and_proposal(proposal_id);

        assert!(proposal.state == ProposalState::Accepted);

        let (wfi, settings) = self.workflow_instance.get(&proposal_id).unwrap();
        let (mut wfi, settings) = (wfi, settings);

        //transition check
        let (transition_id, activity_id): (u8, u8) = wfi
            .get_target_trans_with_act(&wft, ActionIdent::WorkflowAdd)
            .expect("Undefined transition");

        //rights checks
        assert!(self.check_rights(
            &wfs.activity_rights[activity_id as usize - 1].as_slice(),
            &caller
        ));

        let mut bucket = self.storage.get(&settings.storage_key).unwrap();
        let mut args = vec![vec![DataType::U16(workflow_id)]];
        let (result, postprocessing) = wfi.transition_to_next(
            activity_id,
            transition_id,
            &wft,
            &settings,
            args.as_slice(),
            &mut bucket,
            0,
        );

        if settings.obj_validators[activity_id as usize - 1].len() > 0
            && !validate_args(
                &settings.binds,
                &settings.obj_validators[activity_id as usize - 1].as_slice(),
                &settings.validator_exprs.as_slice(),
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
            ActivityResult::Ok => PromiseOrValue::Promise(
                Promise::new(dao_settings.workflow_provider)
                    .function_call(
                        b"get".to_vec(),
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
                        &acc,
                        0,
                        30 * TGAS,
                    )),
            ),
            _ => PromiseOrValue::Value(result),
        };

        self.workflow_instance
            .insert(&proposal_id, &(wfi, settings))
            .unwrap();

        result
    }
}
/* // TODO fix tests
#[cfg(test)]
mod test {
    use library::types::FnCallMetadata;
    use near_sdk::{
        json_types::{U128, U64},
        serde::{Deserialize, Serialize},
        serde_json::{self, Number, Value},
    };

    use crate::action::DataTypeDef;

    /* ---------- TEST OBJECTS ---------- */
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    #[serde(crate = "near_sdk::serde")]
    pub struct TestObject {
        name1: String,
        nullable_obj: Option<InnerNullableTestObj>,
        name2: Vec<String>,
        name3: Vec<U128>,
        obj: InnerTestObject,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    #[serde(crate = "near_sdk::serde")]
    pub struct InnerTestObject {
        nested_1_arr_8: Vec<u8>,
        nested_1_obj: Inner2TestObject,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    #[serde(crate = "near_sdk::serde")]
    pub struct Inner2TestObject {
        nested_2_arr_u64: Vec<U64>,
        bool_val: bool,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    #[serde(crate = "near_sdk::serde")]
    pub struct InnerNullableTestObj {
        test: Option<u8>,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    #[serde(crate = "near_sdk::serde")]
    struct ObjOptCase {
        optional_str: Option<String>,
        optional_obj: Option<ObjOptCaseInner>,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    #[serde(crate = "near_sdk::serde")]
    struct ObjOptCaseInner {
        optional_str: Option<String>,
        vec_u8: Vec<u8>,
    }

    fn get_metadata_case_1() -> Vec<FnCallMetadata> {
        vec![
            FnCallMetadata {
                arg_names: vec!["optional_str".into(), "optional_obj".into()],
                arg_types: vec![DataTypeDef::String(true), DataTypeDef::NullableObject(1)],
            },
            FnCallMetadata {
                arg_names: vec!["optional_str".into(), "vec_u8".into()],
                arg_types: vec![DataTypeDef::String(true), DataTypeDef::VecU8],
            },
        ]
    }

    fn get_names_case_1() -> Vec<Vec<String>> {
        vec![
            vec!["optional_str".into(), "optional_obj".into()],
            vec!["optional_str".into(), "vec_u8".into()],
        ]
    }

    /* ---------- TEST CASES ---------- */

    #[test]
    fn bind_fn_args_optional_case_1() {
        let fncall = FnCallDefinition {
            receiver: "test".into(),
            name: "test".into(),
        };

        let metadata = get_metadata_case_1();
        let names = get_names_case_1();

        let values = vec![
            Some(vec![Value::String("outer_opt_str".into()), Value::Null]),
            Some(vec![
                Value::String("inner_opt_str".into()),
                Value::Array(vec![
                    Value::Number(Number::from(1)),
                    Value::Number(Number::from(2)),
                    Value::Number(Number::from(3)),
                    Value::Number(Number::from(4)),
                    Value::Number(Number::from(5)),
                ]),
            ]),
        ];

        let result_string = fncall.bind_args(&names, &values, &metadata, 0);
        dbg!(result_string.clone());
        let result: ObjOptCase = serde_json::from_str(result_string.as_str()).unwrap();

        let expected_result = ObjOptCase {
            optional_str: Some("outer_opt_str".into()),
            optional_obj: Some(ObjOptCaseInner {
                optional_str: Some("inner_opt_str".into()),
                vec_u8: vec![1, 2, 3, 4, 5],
            }),
        };

        assert_eq!(result, expected_result);

        assert_eq!(
            serde_json::to_string(&result).unwrap(),
            serde_json::to_string(&expected_result).unwrap(),
        );
    }

    #[test]
    fn bind_fn_args_optional_case_2() {
        let fncall = FnCallDefinition {
            receiver: "test".into(),
            name: "test".into(),
        };

        let metadata = get_metadata_case_1();
        let names = get_names_case_1();

        let values = vec![
            Some(vec![Value::Null, Value::Null]),
            Some(vec![
                Value::String("inner_opt_str".into()),
                Value::Array(vec![
                    Value::Number(Number::from(1)),
                    Value::Number(Number::from(2)),
                    Value::Number(Number::from(3)),
                    Value::Number(Number::from(4)),
                    Value::Number(Number::from(5)),
                ]),
            ]),
        ];

        let result_string = fncall.bind_args(&names, &values, &metadata, 0);
        dbg!(result_string.clone());
        let result: ObjOptCase = serde_json::from_str(result_string.as_str()).unwrap();

        let expected_result = ObjOptCase {
            optional_str: None,
            optional_obj: Some(ObjOptCaseInner {
                optional_str: Some("inner_opt_str".into()),
                vec_u8: vec![1, 2, 3, 4, 5],
            }),
        };

        assert_eq!(result, expected_result);

        assert_eq!(
            serde_json::to_string(&result).unwrap(),
            serde_json::to_string(&expected_result).unwrap(),
        );
    }

    #[test]
    fn bind_fn_args_optional_case_3() {
        let fncall = FnCallDefinition {
            receiver: "test".into(),
            name: "test".into(),
        };

        let metadata = get_metadata_case_1();
        let names = get_names_case_1();

        let values = vec![Some(vec![Value::Null, Value::Null]), None];

        let result_string = fncall.bind_args(&names, &values, &metadata, 0);
        dbg!(result_string.clone());
        let result: ObjOptCase = serde_json::from_str(result_string.as_str()).unwrap();

        let expected_result = ObjOptCase {
            optional_str: None,
            optional_obj: None,
        };

        assert_eq!(result, expected_result);

        assert_eq!(
            serde_json::to_string(&result).unwrap(),
            serde_json::to_string(&expected_result).unwrap(),
        );
    }

    #[test]
    fn bind_fn_args_optional_case_4() {
        let fncall = FnCallDefinition {
            receiver: "test".into(),
            name: "test".into(),
        };

        let metadata = get_metadata_case_1();
        let names = get_names_case_1();

        let values = vec![
            Some(vec![Value::String("outer_opt_str".into()), Value::Null]),
            Some(vec![
                Value::Null,
                Value::Array(vec![
                    Value::Number(Number::from(1)),
                    Value::Number(Number::from(2)),
                    Value::Number(Number::from(3)),
                    Value::Number(Number::from(4)),
                    Value::Number(Number::from(5)),
                ]),
            ]),
        ];

        let result_string = fncall.bind_args(&names, &values, &metadata, 0);
        dbg!(result_string.clone());
        let result: ObjOptCase = serde_json::from_str(result_string.as_str()).unwrap();

        let expected_result = ObjOptCase {
            optional_str: Some("outer_opt_str".into()),
            optional_obj: Some(ObjOptCaseInner {
                optional_str: None,
                vec_u8: vec![1, 2, 3, 4, 5],
            }),
        };

        assert_eq!(result, expected_result);

        assert_eq!(
            serde_json::to_string(&result).unwrap(),
            serde_json::to_string(&expected_result).unwrap(),
        );
    }

    #[test]
    fn bind_fn_args_complex() {
        let fncall = FnCallDefinition {
            receiver: "test".into(),
            name: "test".into(),
        };
        let metadata = vec![
            FnCallMetadata {
                arg_names: vec![
                    "name1".into(),
                    "nullable_obj".into(),
                    "name2".into(),
                    "name3".into(),
                    "obj".into(),
                ],
                arg_types: vec![
                    DataTypeDef::String(false),
                    DataTypeDef::NullableObject(1),
                    DataTypeDef::VecString,
                    DataTypeDef::VecU128,
                    DataTypeDef::Object(2),
                ],
            },
            FnCallMetadata {
                arg_names: vec!["test".into()],
                arg_types: vec![DataTypeDef::U8(true)],
            },
            FnCallMetadata {
                arg_names: vec!["nested_1_arr_8".into(), "nested_1_obj".into()],
                arg_types: vec![DataTypeDef::VecU8, DataTypeDef::Object(3)],
            },
            FnCallMetadata {
                arg_names: vec!["nested_2_arr_u64".into(), "bool_val".into()],
                arg_types: vec![DataTypeDef::VecU64, DataTypeDef::Bool(false)],
            },
        ];

        // Inputs
        let names: Vec<Vec<String>> = vec![
            vec![
                "name1".into(),
                "nullable_obj".into(),
                "name2".into(),
                "name3".into(),
                "obj".into(),
            ],
            vec!["test".into()],
            vec!["nested_1_arr_8".into(), "nested_1_obj".into()],
            vec!["nested_2_arr_u64".into(), "bool_val".into()],
        ];

        let values: Vec<Option<Vec<Value>>> = vec![
            Some(vec![
                Value::String("string value".into()),
                Value::Null,
                Value::Array(vec![
                    Value::String("string arr val 1".into()),
                    Value::String("string arr val 2".into()),
                    Value::String("string arr val 3".into()),
                ]),
                Value::Array(vec![
                    "100000000000000000000000000".into(),
                    "200".into(),
                    "300".into(),
                ]),
                Value::Null,
            ]),
            Some(vec![Value::Number(Number::from(77))]),
            Some(vec![Value::Array(vec![
                Value::Number(Number::from(1)),
                Value::Number(Number::from(2)),
                Value::Number(Number::from(3)),
                Value::Number(Number::from(4)),
                Value::Number(Number::from(5)),
                Value::Number(Number::from(255)),
                Value::Number(Number::from(111)),
            ])]),
            Some(vec![
                Value::Array(vec![
                    Value::String("9007199254740993".into()),
                    Value::String("123".into()),
                    Value::String("456".into()),
                ]),
                Value::Bool(true),
            ]),
        ];

        // Test
        let result_string = fncall.bind_args(&names, &values, &metadata, 0);
        dbg!(result_string.clone());
        let result: TestObject = serde_json::from_str(result_string.as_str()).unwrap();

        let expected_result = TestObject {
            name1: "string value".into(),
            nullable_obj: Some(InnerNullableTestObj { test: Some(77) }),
            name2: vec![
                "string arr val 1".into(),
                "string arr val 2".into(),
                "string arr val 3".into(),
            ],
            name3: vec![U128(100000000000000000000000000), U128(200), U128(300)],
            obj: InnerTestObject {
                nested_1_arr_8: vec![1, 2, 3, 4, 5, 255, 111],
                nested_1_obj: Inner2TestObject {
                    nested_2_arr_u64: vec![U64(9007199254740993), U64(123), U64(456)],
                    bool_val: true,
                },
            },
        };
        assert_eq!(result, expected_result);

        assert_eq!(
            serde_json::to_string(&result).unwrap(),
            serde_json::to_string(&expected_result).unwrap(),
        );
    }

}
*/
