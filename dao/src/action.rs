use std::collections::HashMap;

use library::functions::{bind_from_sources, get_value_from_source, serialize_to_json, validate};
use library::storage::StorageBucket;
use library::types::error::ProcessingError;
use library::types::DataType;
use library::workflow::activity::{
    ActionData, ActionInput, Activity, FnCallIdType, TemplateActivity, Terminality,
};
use library::workflow::instance::{Instance, InstanceState};
use library::workflow::settings::{ProposeSettings, TemplateSettings};
use library::workflow::template::Template;
use library::workflow::types::{
    self, ActivityResult, DaoActionIdent, ValidatorType, ValueContainer,
};
use library::{Consts, FnCallId, ObjectValues};
use near_sdk::serde::Serialize;
use near_sdk::{env, near_bindgen, AccountId, Promise};

use crate::callbacks::ext_self;
use crate::constants::{GLOBAL_BUCKET_IDENT, TGAS};
use crate::error::{ActionError, ActivityError};
use crate::group::{GroupInput, GroupMember, GroupSettings, GroupTokenLockInput};
use crate::proposal::{Proposal, ProposalContent, ProposalState};
use crate::settings::DaoSettings;
use crate::token_lock::TokenLock;
use crate::{core::*, group, GroupId, ProposalId, TagCategory, TagId};

#[near_bindgen]
impl Contract {
    /// Testing method
    pub fn run_action(&mut self, action_type: DaoActionIdent, mut args: ObjectValues) {
        self.execute_dao_action(0, action_type, &mut args).unwrap();
    }

    #[payable]
    pub fn wf_run_activity(
        &mut self,
        proposal_id: u32,
        activity_id: usize,
        mut actions_inputs: Vec<Option<ActionInput>>,
    ) -> Result<(), ActivityError> {
        let caller = env::predecessor_account_id();
        let attached_deposit = env::attached_deposit();
        let (proposal, wft, wfs) = self.get_wf_and_proposal(proposal_id);
        let (mut wfi, prop_settings) = self.workflow_instance.get(&proposal_id).unwrap();

        let dao_consts = self.dao_consts();
        let storage_key = prop_settings.storage_key.to_owned();

        let mut storage: Option<StorageBucket> = match storage_key {
            Some(ref key) => self.storage.get(key),
            _ => None,
        };

        let mut global_storage = self.storage.get(&GLOBAL_BUCKET_IDENT.into()).unwrap();

        // TODO: Improve - as it cannot be passed down to inner functions.
        let sources = ValueContainer {
            dao_consts: &dao_consts,
            tpl_consts: &wft.constants,
            settings_consts: &wfs.constants,
            activity_shared_consts: None,
            action_proposal_consts: None,
            storage: storage.as_mut(),
            global_storage: &mut global_storage,
        };

        // Check states
        assert!(
            proposal.state == ProposalState::Accepted,
            "Proposal is not accepted."
        );
        assert!(
            wfi.state == InstanceState::Running,
            "Workflow is not running."
        );

        // Loop / other activity case
        if wfi.current_activity_id as usize == activity_id
            && wfi.actions_done_count == wfi.actions_total
            || wfi.current_activity_id as usize != activity_id
        {
            // Find transition
            let transition = wfi
                .find_transition(&wft, activity_id)
                .expect("Transition is not possible.");

            // Check transition counter
            assert!(
                wfi.check_transition_counter(
                    activity_id as usize,
                    wfs.transition_limits.as_slice()
                ),
                "Reached transition limit."
            );

            // Check transition condition
            assert!(
                transition
                    .cond
                    .as_ref()
                    .map(|expr| expr
                        .bind_and_eval(&sources, &[])
                        .expect("Binding and eval transition condition failed.")
                        .try_into_bool()
                        .expect("Invalid transition condition definition."))
                    .unwrap_or(true),
                "Transition condition failed."
            );
        }

        let is_dao_activity = wft.activities[activity_id].is_dao_activity();

        // Finds activity
        let activity = wft
            .activities
            .get(activity_id)
            .expect("Activity does not exists.")
            .activity_as_ref()
            .unwrap();

        // Skip right checks for automatic activity.
        if !activity.automatic {
            // Check rights
            assert!(
                self.check_rights(&wfs.activity_rights[activity_id].as_slice(), &caller),
                "No rights."
            );
        }

        // Check action input structure
        assert!(
            self.check_activity_input(
                activity.actions.as_slice(),
                actions_inputs.as_slice(),
                wfi.actions_done_count as usize
            ),
            "Activity input structure is invalid."
        );

        let actions_done_before = wfi.actions_done_count;
        let activity_terminality = activity.terminal;
        let _activity_postprocessing = activity.postprocessing.clone(); // TODO: Solve.
        let result;

        if is_dao_activity {
            result = self.run_dao_activity(
                caller,
                attached_deposit,
                proposal_id,
                wft,
                wfs,
                activity_id,
                &mut wfi,
                actions_inputs,
                &prop_settings,
                dao_consts,
                storage.as_mut(),
                &mut global_storage,
            );
        } else {
            result = self.run_fncall_activity(
                proposal_id,
                wft,
                wfs,
                activity_id,
                &mut wfi,
                actions_inputs,
                &prop_settings,
                dao_consts,
                storage.as_mut(),
                &mut global_storage,
            );
        }

        let actions_done_after = wfi.actions_done_count;

        // TODO: Discuss Activity postprocessing when all actions are DONE.

        let result = match result {
            Err(e) => {
                // We want to make the underlying transaction succeed if at least one of the action was successfuly executed.
                if actions_done_before == actions_done_after {
                    panic!("Not a single action was executed.");
                } else {
                    let e = ActivityError::from(e);

                    if e.is_fatal() {
                        wfi.state = InstanceState::FatalError;
                    }

                    Err(e)
                }
            }
            _ => {
                if activity_terminality == Terminality::Automatic
                    && wfi.actions_done_count == wfi.actions_total
                {
                    wfi.state = InstanceState::Finished;
                }

                Ok(())
            }
        };

        // Save mutated storage.
        if let Some(storage) = storage {
            self.storage.insert(&storage_key.unwrap(), &storage);
        }
        self.storage
            .insert(&GLOBAL_BUCKET_IDENT.into(), &global_storage);

        // Save mutated instance state.
        self.workflow_instance
            .insert(&proposal_id, &(wfi, prop_settings));

        result
    }
}

// Internal action methods.
impl Contract {
    /// Tries to run all activity's actions.
    /// Some checks must be done before calling this function.
    pub fn run_dao_activity(
        &mut self,
        caller: AccountId,
        mut attached_deposit: u128,
        proposal_id: u32,
        mut template: Template,
        template_settings: TemplateSettings,
        activity_id: usize,
        instance: &mut Instance,
        mut actions_inputs: Vec<Option<ActionInput>>,
        prop_settings: &ProposeSettings,
        dao_consts: Box<Consts>,
        storage: Option<&mut StorageBucket>,
        global_storage: &mut StorageBucket,
    ) -> Result<(), ActionError> {
        // Assumption activity_id is valid activity_id index.
        let mut activity = match template.activities.swap_remove(activity_id) {
            Activity::DaoActivity(a) => a,
            _ => return Err(ActionError::InvalidWfStructure),
        };

        let mut sources = ValueContainer {
            dao_consts: &dao_consts,
            tpl_consts: &template.constants,
            settings_consts: &template_settings.constants,
            activity_shared_consts: None,
            action_proposal_consts: None,
            storage,
            global_storage,
        };

        // Loop which tries to execute all actions, starting from the last done. Returns when something goes wrong.
        // Assuming that structure of inputs was checked above therefore unwraping on indexes is OK.
        for idx in instance.actions_done_count as usize..activity.actions.len() {
            let action = match actions_inputs.get_mut(idx).unwrap() {
                Some(a) => a,
                None => continue, // skip optional actions
            };

            let tpl_action = activity.actions.get_mut(idx).unwrap();

            // Check exec condition
            match tpl_action.exec_condition.as_ref() {
                Some(cond) => {
                    if !cond.bind_and_eval(&sources, &[])?.try_into_bool()? {
                        return Err(ActionError::Condition(idx as u8));
                    }
                }
                None => (),
            };

            // Assign current action proposal binds to source.
            if let Some(binds) = &prop_settings.binds[activity_id] {
                sources.activity_shared_consts = Some(&binds.shared);
                sources.action_proposal_consts = Some(&binds.values[idx]);
            } else {
                sources.activity_shared_consts = None;
                sources.action_proposal_consts = None;
            }

            let action_data = std::mem::replace(&mut tpl_action.action_data, ActionData::None)
                .try_into_action_data()
                .ok_or_else(|| ActionError::InvalidWfStructure)?;

            // Need metadata coz validations and bindings. Metadata are always included in DAO.
            let (metadata, input_defs) = (
                self.dao_action_metadata.get(&action_data.name).unwrap(),
                action_data.inputs_definitions.as_slice(),
            );

            // Check input validators
            if tpl_action.input_validators.len() > 0 {
                if !validate(
                    &sources,
                    tpl_action.input_validators.as_slice(),
                    template.validator_exprs.as_slice(),
                    metadata.as_slice(),
                    action.values.as_slice(),
                )? {
                    return Err(ActionError::Validation(idx as u8));
                }
            }

            // Bind DaoAction
            bind_from_sources(
                input_defs,
                &sources,
                template.expressions.as_slice(),
                &mut action.values,
                0,
            )?;

            if action_data.name == DaoActionIdent::Event {
                let deposit = match &action_data.required_deposit {
                    Some(arg_src) => get_value_from_source(arg_src, &sources)
                        .map_err(|_| ActionError::InvalidSource)?
                        .try_into_u128()?,
                    _ => 0,
                };

                attached_deposit = attached_deposit
                    .checked_sub(deposit)
                    .ok_or(ActionError::NotEnoughDeposit)?;

                // Insert caller into 0th position.
                action
                    .values
                    .get_mut(0)
                    .ok_or(ActionError::InputStructure(0))?
                    .insert(0, DataType::String(caller.clone()));
            } else {
                self.execute_dao_action(proposal_id, action_data.name, &mut action.values)?;
            }

            // TODO: Handle error so we do only part of the batch.
            if let Some(mut pp) = tpl_action.postprocessing.take() {
                pp.bind_and_convert(&sources, &mut action.values)
                    .map_err(|_| ActionError::ActionPostprocessing(idx as u8))?;
                // TODO: Different execute version for DaoActions?
                if pp
                    .execute(
                        vec![],
                        &mut sources.storage,
                        sources.global_storage,
                        &mut None,
                    )
                    .is_err()
                {
                    return Err(ActionError::ActionPostprocessing(idx as u8));
                }
            }

            instance.actions_done_count += 1;
        }

        Ok(())
    }
    /// FnCall version of `run_dao_activity` function.
    pub fn run_fncall_activity(
        &mut self,
        proposal_id: u32,
        mut template: Template,
        template_settings: TemplateSettings,
        activity_id: usize,
        instance: &mut Instance,
        mut actions_inputs: Vec<Option<ActionInput>>,
        prop_settings: &ProposeSettings,
        dao_consts: Box<Consts>,
        storage: Option<&mut StorageBucket>,
        global_storage: &mut StorageBucket,
    ) -> Result<(), ActionError> {
        // Assumption activity_id is valid activity_id index.
        let mut activity = match template.activities.swap_remove(activity_id) {
            Activity::FnCallActivity(a) => a,
            _ => return Err(ActionError::InvalidWfStructure),
        };

        let mut sources = ValueContainer {
            dao_consts: &dao_consts,
            tpl_consts: &template.constants,
            settings_consts: &template_settings.constants,
            activity_shared_consts: None,
            action_proposal_consts: None,
            storage,
            global_storage,
        };

        // Loop which tries to execute all actions, starting from the last done. Returns when something goes wrong.
        // Assuming that structure of inputs was checked above therefore unwraping on indexes is OK.
        for idx in instance.actions_done_count as usize..activity.actions.len() {
            let action = match actions_inputs.get_mut(idx).unwrap() {
                Some(a) => a,
                None => continue, // skip optional actions
            };

            let tpl_action = activity.actions.get_mut(idx).unwrap();

            // Check exec condition
            match tpl_action.exec_condition.as_ref() {
                Some(cond) => {
                    if !cond.bind_and_eval(&sources, &[])?.try_into_bool()? {
                        return Err(ActionError::Condition(idx as u8));
                    }
                }
                None => (),
            };

            // Assign current action proposal binds to source.
            if let Some(binds) = &prop_settings.binds[activity_id] {
                sources.activity_shared_consts = Some(&binds.shared);
                sources.action_proposal_consts = Some(&binds.values[idx]);
            } else {
                sources.activity_shared_consts = None;
                sources.action_proposal_consts = None;
            }

            let action_data = std::mem::replace(&mut tpl_action.action_data, ActionData::None)
                .try_into_fncall_data()
                .ok_or_else(|| ActionError::InvalidWfStructure)?;

            // TODO: Reduce cloning.
            // Metadata are provided by workflow provider when workflow is added. Missing metadata are fault of the workflow provider and are considered as fatal runtime error.
            let (name, method, metadata) = match action_data.id {
                FnCallIdType::Static((account, method)) => {
                    if account == "self" {
                        let name = env::current_account_id();
                        (
                            name.clone(),
                            method.clone(),
                            self.function_call_metadata
                                .get(&(name.clone(), method.clone()))
                                .ok_or_else(|| ActionError::MissingFnCallMetadata(method))?,
                        )
                    } else {
                        (
                            account.clone(),
                            method.clone(),
                            self.function_call_metadata
                                .get(&(account, method.clone()))
                                .ok_or_else(|| ActionError::MissingFnCallMetadata(method))?,
                        )
                    }
                }
                FnCallIdType::Dynamic(arg_src, method) => {
                    let name = get_value_from_source(&arg_src, &sources)
                        .map_err(|e| ProcessingError::Source(e))?
                        .try_into_string()?;
                    (
                        name.clone(),
                        method.clone(),
                        self.function_call_metadata
                            .get(&(name.clone(), method.clone()))
                            .ok_or_else(|| ActionError::MissingFnCallMetadata(method))?,
                    )
                }
                FnCallIdType::StandardStatic((account, method)) => {
                    if account == "self" {
                        let name = env::current_account_id();
                        (
                            name.clone(),
                            method.clone(),
                            self.standard_function_call_metadata
                                .get(&method.clone())
                                .ok_or_else(|| ActionError::MissingFnCallMetadata(method))?,
                        )
                    } else {
                        (
                            account.clone(),
                            method.clone(),
                            self.function_call_metadata
                                .get(&(account, method.clone()))
                                .ok_or_else(|| ActionError::MissingFnCallMetadata(method))?,
                        )
                    }
                }
                FnCallIdType::StandardDynamic(arg_src, method) => {
                    let name = get_value_from_source(&arg_src, &sources)
                        .map_err(|e| ProcessingError::Source(e))?
                        .try_into_string()?;
                    (
                        name.clone(),
                        method.clone(),
                        self.standard_function_call_metadata
                            .get(&name)
                            .ok_or_else(|| ActionError::MissingFnCallMetadata(method))?,
                    )
                }
            };

            let input_defs = action_data.inputs_definitions.as_slice();

            // Check input validators
            if tpl_action.input_validators.len() > 0 {
                if !validate(
                    &sources,
                    tpl_action.input_validators.as_slice(),
                    template.validator_exprs.as_slice(),
                    metadata.as_slice(),
                    action.values.as_slice(),
                )? {
                    return Err(ActionError::Validation(idx as u8));
                }
            }

            // Bind DaoAction
            bind_from_sources(
                input_defs,
                &sources,
                template.expressions.as_slice(),
                &mut action.values,
                0,
            )?;

            let deposit = match action_data.deposit {
                Some(arg_src) => get_value_from_source(&arg_src, &sources)
                    .map_err(|_| ActionError::InvalidSource)?
                    .try_into_u128()?,
                None => 0,
            };

            let args = serialize_to_json(action.values.as_slice(), metadata.as_slice(), 0);

            let pp = if let Some(mut pp) = tpl_action.postprocessing.take() {
                pp.bind_and_convert(&sources, &mut action.values)
                    .map_err(|_| ActionError::ActionPostprocessing(idx as u8))?;
                Some(pp)
            } else {
                None
            };

            // Dispatch fncall and its postprocessing.
            Promise::new(name)
                .function_call(
                    method.clone().into_bytes(),
                    args.into_bytes(),
                    deposit,
                    action_data.tgas as u64 * 10u64.pow(12),
                )
                .then(ext_self::postprocess(
                    proposal_id,
                    activity_id as u8,
                    idx as u8,
                    prop_settings.storage_key.clone(),
                    pp,
                    tpl_action.must_succeed,
                    activity.terminal == Terminality::Automatic,
                    &env::current_account_id(),
                    0,
                    50 * 10u64.pow(12),
                ));

            instance.actions_done_count += 1;
        }

        Ok(())
    }

    pub fn group_add(&mut self, group: GroupInput) -> bool {
        self.add_group(group);

        true
    }
    pub fn group_remove(&mut self, id: GroupId) -> bool {
        match self.groups.remove(&id) {
            Some(mut group) => {
                let token_lock: TokenLock = group.remove_storage_data().into();
                self.ft_total_locked -= token_lock.amount - token_lock.distributed;
                self.total_members_count -= group.members.members_count() as u32;
            }
            _ => (),
        }

        true
    }

    pub fn group_update(&mut self, id: GroupId, settings: GroupSettings) -> bool {
        match self.groups.get(&id) {
            Some(mut group) => {
                group.settings = settings;
                self.groups.insert(&id, &group);
            }
            _ => (),
        }

        true
    }

    pub fn group_add_members(&mut self, id: GroupId, members: Vec<GroupMember>) -> bool {
        match self.groups.get(&id) {
            Some(mut group) => {
                self.total_members_count += group.add_members(members);
                self.groups.insert(&id, &group);
            }
            _ => (),
        }
        true
    }

    pub fn group_remove_member(&mut self, id: GroupId, member: AccountId) -> bool {
        match self.groups.get(&id) {
            Some(mut group) => {
                group.remove_member(member);
                self.total_members_count -= 1;
                self.groups.insert(&id, &group);
            }
            _ => (),
        }

        true
    }

    pub fn settings_update(&mut self, settings: DaoSettings) {
        //assert_valid_dao_settings(&settings);
        self.settings.replace(&settings.into());
    }

    /// Returns tuple of start, end index for the new tags
    pub fn tag_add(
        &mut self,
        proposal_id: ProposalId,
        category: TagCategory,
        tags: Vec<String>,
    ) -> Option<(TagId, TagId)> {
        unimplemented!();
        //let mut t = self.tags.get(&category).unwrap_or(Tags::new());
        //let ids = t.insert(tags);
        //self.tags.insert(&category, &t);
    }

    pub fn tag_edit(
        &mut self,
        proposal_id: ProposalId,
        category: TagCategory,
        id: TagId,
        value: String,
    ) -> bool {
        unimplemented!();

        //match self.tags.get(&category) {
        //    Some(mut t) => {
        //        t.rename(id, value);
        //        self.tags.insert(&category, &t);
        //    }
        //}
        //true
    }

    pub fn tag_remove(
        &mut self,
        proposal_id: ProposalId,
        category: TagCategory,
        id: TagId,
    ) -> bool {
        unimplemented!();

        //match self.tags.get(&category) {
        //    Some(mut t) => {
        //        t.remove(id);
        //        self.tags.insert(&category, &t);
        //    }
        //    None => (),
        //}
        //true
    }

    /// Internally sends `group_id`'s FT `amount` to the `account_ids`.
    pub fn ft_distribute(
        &mut self,
        group_id: u16,
        amount: u32,
        account_ids: Vec<AccountId>,
    ) -> bool {
        if let Some(mut group) = self.groups.get(&group_id) {
            match group.distribute_ft(amount) && account_ids.len() > 0 {
                true => {
                    self.groups.insert(&group_id, &group);
                    self.distribute_ft(amount, &account_ids);
                }
                _ => (),
            }
        }

        true
    }

    // TODO: Move to standard fncalls.
    pub fn treasury_send_near(&mut self, receiver_id: AccountId, amount: u128) -> bool {
        Promise::new(receiver_id).transfer(amount);
        true
    }

    // TODO: Move to standard fncalls.
    pub fn treasury_send_ft(
        &mut self,
        ft_account_id: AccountId,
        receiver_id: AccountId,
        amount: u128,
        memo: Option<String>,
        msg: Option<String>,
        is_contract: bool,
    ) -> bool {
        if is_contract {
            Promise::new(ft_account_id).function_call(
                b"ft_transfer_call".to_vec(),
                format!(
                    "{{\"receiver_id\":\"{}\",\"amount\":\"{}\",\"memo\":\"{}\",\"msg\":\"{}\"}}",
                    receiver_id,
                    amount,
                    memo.unwrap_or("null".into()),
                    msg.unwrap_or_default(),
                )
                .as_bytes()
                .to_vec(),
                1,
                30 * TGAS,
            );
        } else {
            Promise::new(ft_account_id).function_call(
                b"ft_transfer".to_vec(),
                format!(
                    "{{\"receiver_id\":\"{}\",\"amount\":\"{}\",\"memo\":\"{}\"}}",
                    receiver_id,
                    amount,
                    memo.unwrap_or("null".into()),
                )
                .as_bytes()
                .to_vec(),
                1,
                15 * TGAS,
            );
        }

        true
    }

    // TODO: Move to standard fncalls.
    pub fn treasury_send_nft(
        &mut self,
        nft_account_id: AccountId,
        receiver_id: String,
        nft_id: String,
        memo: Option<String>,
        approval_id: u32,
        msg: Option<String>,
        is_contract: bool,
    ) -> bool {
        if is_contract {
            Promise::new(nft_account_id).function_call(b"nft_transfer_call".to_vec(),
    format!(
                "{{\"receiver_id\":\"{}\",\"token_id\":\"{}\",\"approval_id\":{},\"memo\":\"{}\",\"msg\":\"{}\"}}",
                receiver_id,
                nft_id,
                approval_id,
                memo.unwrap_or("null".into()),
                msg.unwrap_or_default(),
                )
                .as_bytes()
                .to_vec(),
                1,
                40 * TGAS
            );
        } else {
            Promise::new(nft_account_id).function_call(b"nft_transfer_call".to_vec(),
        format!(
                    "{{\"receiver_id\":\"{}\",\"token_id\":\"{}\",\"approval_id\":{},\"memo\":\"{}\"}}",
                    receiver_id,
                    nft_id,
                    approval_id,
                    memo.unwrap_or("null".into()),
                    )
                    .as_bytes()
                    .to_vec(),
                    1,
                    20 * TGAS
                );
        }

        true
    }
}
