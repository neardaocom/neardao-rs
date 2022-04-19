use library::functions::binding::{bind_from_sources, get_value_from_source};
use library::functions::serialization::serialize_to_json;
use library::functions::validation::validate;
use library::storage::StorageBucket;
use library::types::datatype::Value;
use library::types::error::ProcessingError;
use library::workflow::activity::{ActionData, ActionInput, Activity, FnCallIdType, Terminality};
use library::workflow::instance::InstanceState;
use library::workflow::settings::{ProposeSettings, TemplateSettings};
use library::workflow::template::Template;
use library::workflow::types::{DaoActionIdent, ValueContainer};
use library::{Consts, ObjectValues};
use near_sdk::{env, near_bindgen, AccountId, Gas, Promise};

use crate::callbacks::ext_self;
use crate::constants::GLOBAL_BUCKET_IDENT;
use crate::error::{ActionError, ActivityError};
use crate::group::{GroupInput, GroupMember, GroupSettings};
use crate::proposal::ProposalState;
use crate::settings::{assert_valid_dao_settings, DaoSettings};
use crate::tags::Tags;
use crate::token_lock::TokenLock;
use crate::{core::*, GroupId, TagId};

#[near_bindgen]
impl Contract {
    /// Testing method
    #[allow(unused_mut)]
    pub fn run_action(&mut self, action_type: DaoActionIdent, mut args: ObjectValues) {
        self.execute_dao_action(0, action_type, &mut args).unwrap();
    }

    // TODO: Auto-finish WF then there is no other possible transition regardless terminality.
    #[allow(unused_mut)]
    #[allow(clippy::nonminimal_bool)]
    #[payable]
    #[handle_result]
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
        let mut is_new_transition = false;

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

            is_new_transition = true;
        }

        let is_dao_activity = wft.activities[activity_id].is_dao_activity();

        // Finds activity
        let activity = wft
            .activities
            .get(activity_id)
            .expect("Activity does not exists.")
            .activity_as_ref()
            .expect("Activity is invalid.");

        // Skip right checks for automatic activity.
        // TODO: This might be solved by settings run rights "Anyone" to the automatic activity.
        if !activity.automatic {
            // Check rights
            assert!(
                self.check_rights(wfs.activity_rights[activity_id].as_slice(), &caller),
                "No rights."
            );
        }

        // Check action input structure including optional actions.
        assert!(
            self.check_activity_input(
                activity.actions.as_slice(),
                actions_inputs.as_slice(),
                wfi.actions_done_count as usize
            ),
            "Activity input structure is invalid."
        );

        // TODO: This will be refactored, its just tmp solution because of Rust move semantics.
        let actions_done_before = wfi.actions_done_count;
        let mut actions_done_new = wfi.actions_done_count;
        let activity_actions_count = activity.actions.len() as u8;
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
                &mut actions_done_new,
                actions_inputs,
                &prop_settings,
                dao_consts,
                storage.as_mut(),
                &mut global_storage,
            );

            // TODO: Discuss Activity postprocessing when all actions are DONE.

            // In case not a single DaoAction was executed, then consider this call as failed and panic!
            if result.is_err() && actions_done_before == actions_done_new {
                panic!("Not a single action was executed.");
            } else {
                //At least one action was executed.
                wfi.last_transition_done_at = env::block_timestamp() / 10u64.pow(9);

                if is_new_transition {
                    wfi.transition_next(
                        activity_id as u8,
                        activity_actions_count,
                        actions_done_new,
                    );
                } else {
                    wfi.actions_done_count = actions_done_new;
                }

                // This can happen only if no error occured.
                if wfi.actions_done_count == wfi.actions_total
                    && activity_terminality == Terminality::Automatic
                {
                    wfi.state = InstanceState::Finished;
                }

                // Save mutated storage.
                if let Some(storage) = storage {
                    self.storage.insert(&storage_key.unwrap(), &storage);
                }
                self.storage
                    .insert(&GLOBAL_BUCKET_IDENT.into(), &global_storage);
            }
        } else {
            // Optional actions will be immediatelly added to instance.
            let mut optional_actions = 0;
            // In case of fn calls activity storages are mutated in postprocessing as promises resolve.
            result = self.run_fncall_activity(
                proposal_id,
                wft,
                wfs,
                activity_id,
                &mut actions_done_new,
                &mut optional_actions,
                actions_inputs,
                &prop_settings,
                dao_consts,
                storage.as_mut(),
                &mut global_storage,
            );

            if result.is_err() && actions_done_before == actions_done_new {
                panic!("Not a single action was executed.");
            } else {
                wfi.actions_done_count += optional_actions;
                wfi.set_new_awaiting_state(
                    activity_id as u8,
                    is_new_transition,
                    activity_actions_count,
                    activity_terminality == Terminality::Automatic,
                );
            }
        }

        // Decide if is fatal error.
        let result: Result<(), ActivityError> = if let Err(e) = result {
            let e = ActivityError::from(e);

            if e.is_fatal() {
                wfi.state = InstanceState::FatalError;
            }
            Err(e)
        } else {
            Ok(())
        };

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
    #[allow(clippy::too_many_arguments)]
    pub fn run_dao_activity(
        &mut self,
        caller: AccountId,
        mut attached_deposit: u128,
        proposal_id: u32,
        mut template: Template,
        template_settings: TemplateSettings,
        activity_id: usize,
        actions_done_count: &mut u8,
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
        let last_action_done = *actions_done_count as usize;
        for idx in last_action_done..activity.actions.len() {
            let action = match actions_inputs.get_mut(idx).unwrap() {
                Some(a) => a,
                None => {
                    *actions_done_count += 1;
                    continue;
                }
            };

            let tpl_action = activity.actions.get_mut(idx).unwrap();

            // Check exec condition
            if let Some(cond) = tpl_action.exec_condition.as_ref() {
                if !cond.bind_and_eval(&sources, &[])?.try_into_bool()? {
                    return Err(ActionError::Condition(idx as u8));
                }
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
                .ok_or(ActionError::InvalidWfStructure)?;

            // Need metadata coz validations and bindings. Metadata are always included in DAO.
            let (metadata, input_defs) = (
                self.dao_action_metadata.get(&action_data.name).unwrap(),
                action_data.inputs_definitions.as_slice(),
            );

            // Check input validators
            if !tpl_action.input_validators.is_empty()
                && !validate(
                    &sources,
                    tpl_action.input_validators.as_slice(),
                    template.validator_exprs.as_slice(),
                    metadata.as_slice(),
                    action.values.as_slice(),
                )?
            {
                return Err(ActionError::Validation(idx as u8));
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
                    .insert(0, Value::String(caller.to_string()));
            } else {
                self.execute_dao_action(proposal_id, action_data.name, &mut action.values)?;
            }

            // TODO: Handle error so we do only part of the batch.
            if let Some(mut pp) = tpl_action.postprocessing.take() {
                pp.bind_instructions(&sources, action.values.as_slice())
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

            *actions_done_count += 1;
        }

        Ok(())
    }
    /// FnCall version of `run_dao_activity` function.
    #[allow(clippy::too_many_arguments)]
    pub fn run_fncall_activity(
        &mut self,
        proposal_id: u32,
        mut template: Template,
        template_settings: TemplateSettings,
        activity_id: usize,
        actions_done_count: &mut u8,
        optional_actions: &mut u8,
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
        let last_action_done = *actions_done_count as usize;

        // This strange variable is here because "optional-required-optional" actions case might happen. Therefore we must not considered 3th action as sucessfull but instead of that break the cycle.
        // This might be redundant and "YAGNI" stuff I let it stay here for now.
        let mut optional_state = 0;
        for idx in last_action_done..activity.actions.len() {
            let action = match actions_inputs.get_mut(idx).unwrap() {
                Some(a) => {
                    if optional_state == 1 {
                        optional_state = 2;
                    }
                    a
                }
                None => {
                    if optional_state == 2 {
                        break;
                    }
                    optional_state = 1;
                    *optional_actions += 1;
                    *actions_done_count += 1;
                    continue;
                }
            };

            let tpl_action = activity.actions.get_mut(idx).unwrap();

            // Check exec condition
            if let Some(cond) = tpl_action.exec_condition.as_ref() {
                if !cond.bind_and_eval(&sources, &[])?.try_into_bool()? {
                    return Err(ActionError::Condition(idx as u8));
                }
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
                .ok_or(ActionError::InvalidWfStructure)?;

            // TODO: Reduce cloning.
            // Metadata are provided by workflow provider when workflow is added. Missing metadata are fault of the workflow provider and are considered as fatal runtime error.
            let (name, method, metadata) = match action_data.id {
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
                    let name = get_value_from_source(&arg_src, &sources)
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
                    let name = get_value_from_source(&arg_src, &sources)
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

            let input_defs = action_data.inputs_definitions.as_slice();

            // Check input validators
            if !tpl_action.input_validators.is_empty()
                && !validate(
                    &sources,
                    tpl_action.input_validators.as_slice(),
                    template.validator_exprs.as_slice(),
                    metadata.as_slice(),
                    action.values.as_slice(),
                )?
            {
                return Err(ActionError::Validation(idx as u8));
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
                pp.bind_instructions(&sources, action.values.as_slice())
                    .map_err(|_| ActionError::ActionPostprocessing(idx as u8))?;
                Some(pp)
            } else {
                None
            };

            // Dispatch fncall and its postprocessing.
            Promise::new(name)
                .function_call(
                    method,
                    args.into_bytes(),
                    deposit,
                    Gas(action_data.tgas as u64 * 10u64.pow(12)),
                )
                .then(ext_self::postprocess(
                    proposal_id,
                    idx as u8,
                    tpl_action.must_succeed,
                    prop_settings.storage_key.clone(),
                    pp,
                    env::current_account_id(),
                    0,
                    Gas(50 * 10u64.pow(12)),
                ));

            // We need number of successfully dispatched promises.
            *actions_done_count += 1;
        }

        Ok(())
    }

    pub fn group_add(&mut self, group: GroupInput) -> bool {
        self.add_group(group);

        true
    }
    pub fn group_remove(&mut self, id: GroupId) -> bool {
        if let Some(mut group) = self.groups.remove(&id) {
            let token_lock: TokenLock = group.remove_storage_data();
            self.ft_total_locked -= token_lock.amount - token_lock.distributed;
            self.total_members_count -= group.members.members_count() as u32;
            true
        } else {
            false
        }
    }

    pub fn group_update(&mut self, id: GroupId, settings: GroupSettings) -> bool {
        if let Some(mut group) = self.groups.get(&id) {
            group.settings = settings;
            self.groups.insert(&id, &group);
            true
        } else {
            false
        }
    }

    pub fn group_add_members(&mut self, id: GroupId, members: Vec<GroupMember>) -> bool {
        if let Some(mut group) = self.groups.get(&id) {
            self.total_members_count += group.add_members(members);
            self.groups.insert(&id, &group);
            true
        } else {
            false
        }
    }

    pub fn group_remove_member(&mut self, id: GroupId, member: AccountId) -> bool {
        if let Some(mut group) = self.groups.get(&id) {
            group.remove_member(member);
            self.total_members_count -= 1;
            self.groups.insert(&id, &group);
            true
        } else {
            false
        }
    }

    pub fn settings_update(&mut self, settings: DaoSettings) {
        assert_valid_dao_settings(&settings);
        self.settings.replace(&settings.into());
    }

    /// Returns tuple of start, end index for the new tags
    pub fn tag_add(&mut self, category: String, tags: Vec<String>) -> Option<(TagId, TagId)> {
        let mut t = self.tags.get(&category).unwrap_or_else(Tags::new);
        let ids = t.insert(tags);
        self.tags.insert(&category, &t);
        ids
    }

    pub fn tag_edit(&mut self, category: String, id: u16, value: String) -> bool {
        match self.tags.get(&category) {
            Some(mut t) => {
                t.rename(id, value);
                self.tags.insert(&category, &t);
                true
            }
            None => false,
        }
    }

    pub fn tag_remove(&mut self, category: String, id: u16) -> bool {
        match self.tags.get(&category) {
            Some(mut t) => {
                t.remove(id);
                self.tags.insert(&category, &t);
                true
            }
            None => false,
        }
    }

    /// Internally sends `group_id`'s FT `amount` to the `account_ids`.
    pub fn ft_distribute(
        &mut self,
        group_id: u16,
        amount: u32,
        account_ids: Vec<AccountId>,
    ) -> bool {
        if let Some(mut group) = self.groups.get(&group_id) {
            if group.distribute_ft(amount) && !account_ids.is_empty() {
                self.groups.insert(&group_id, &group);
                self.distribute_ft(amount, &account_ids);
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    // TODO: Refactoring
    /*     pub fn treasury_send_near(&mut self, receiver_id: AccountId, amount: u128) -> bool {
        Promise::new(receiver_id).transfer(amount);
        true
    }

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
                    memo.unwrap_or_else(|| "null".into()),
                    msg.unwrap_or_default(),
                )
                .as_bytes()
                .to_vec(),
                1,
                Gas(30 * TGAS),
            );
        } else {
            Promise::new(ft_account_id).function_call(
                b"ft_transfer".to_vec(),
                format!(
                    "{{\"receiver_id\":\"{}\",\"amount\":\"{}\",\"memo\":\"{}\"}}",
                    receiver_id,
                    amount,
                    memo.unwrap_or_else(|| "null".into()),
                )
                .as_bytes()
                .to_vec(),
                1,
                Gas(15 * TGAS),
            );
        }

        true
    }

    #[allow(clippy::too_many_arguments)]
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
                memo.unwrap_or_else(|| "null".into()),
                msg.unwrap_or_default(),
                )
                .as_bytes()
                .to_vec(),
                1,
                Gas(40 * TGAS)
            );
        } else {
            Promise::new(nft_account_id).function_call(b"nft_transfer_call".to_vec(),
        format!(
                    "{{\"receiver_id\":\"{}\",\"token_id\":\"{}\",\"approval_id\":{},\"memo\":\"{}\"}}",
                    receiver_id,
                    nft_id,
                    approval_id,
                    memo.unwrap_or_else(|| "null".into()),
                    )
                    .as_bytes()
                    .to_vec(),
                    1,
                    Gas(20 * TGAS)
                );
        }

        true
    } */
}
