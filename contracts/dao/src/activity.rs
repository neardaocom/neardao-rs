use library::functions::serialization::serialize_to_json;
use library::functions::validation::validate;
use library::functions::{binding::bind_input, utils::get_value_from_source};
use library::interpreter::expression::EExpr;
use library::MethodName;

use library::storage::StorageBucket;
use library::types::activity_input::ActivityInput;
use library::types::error::ProcessingError;
use library::types::source::{DefaultSource, Source};
use library::workflow::action::{ActionInput, ActionType, FnCallIdType, TemplateAction};
use library::workflow::activity::{TemplateActivity, Terminality};
use library::workflow::instance::InstanceState;
use library::workflow::postprocessing::Postprocessing;
use library::workflow::settings::TemplateSettings;
use library::workflow::template::Template;
use library::workflow::types::ArgSrc::User;
use library::workflow::types::{ActivityRight, DaoActionIdent, ObjectMetadata};
use near_sdk::{env, ext_contract, log, near_bindgen, AccountId, Gas, Promise, PromiseResult};

use crate::constants::GLOBAL_BUCKET_IDENT;
use crate::core::*;
use crate::error::{ActionError, ActivityError, ERR_PROMISE_INVALID_RESULTS_COUNT};
use crate::helper::deserialize::{
    try_bind_accounts, try_bind_group, try_bind_group_members, try_bind_partition, try_bind_reward,
    try_to_bind_member_roles, try_to_bind_roles,
};
use crate::internal::utils::current_timestamp_sec;
use crate::internal::ActivityContext;
use crate::proposal::ProposalState;
use crate::reward::RewardActivity;
use crate::role::UserRoles;

#[ext_contract(ext_self)]
trait ExtActivity {
    fn postprocess(
        &mut self,
        instance_id: u32,
        action_id: u8,
        must_succeed: bool,
        storage_key: Option<String>,
        postprocessing: Option<Postprocessing>,
    ) -> ActivityResult;
}

#[near_bindgen]
impl Contract {
    // TODO: Auto-finish WF then there is no other possible transition regardless terminality.
    #[payable]
    pub fn workflow_run_activity(
        &mut self,
        proposal_id: u32,
        activity_id: usize,
        actions_inputs: Vec<Option<ActionInput>>,
    ) -> Option<ActivityError> {
        let (proposal, wft, wfs) = self.get_workflow_and_proposal(proposal_id);
        let mut wfi = self.workflow_instance.get(&proposal_id).unwrap();
        let mut prop_settings = self.workflow_propose_settings.get(&proposal_id).unwrap();
        let dao_consts = self.dao_consts();

        let Template {
            mut activities,
            constants,
            transitions,
            expressions,
            ..
        } = wft;

        let TemplateSettings {
            activity_rights, ..
        } = wfs;
        let storage_key = prop_settings.storage_key.to_owned();
        let storage: Option<StorageBucket> = match storage_key {
            Some(ref key) => self.storage.get(key),
            _ => None,
        };
        let global_storage = self.storage.get(&GLOBAL_BUCKET_IDENT.into()).unwrap();
        let mut sources: Box<dyn Source> = Box::new(DefaultSource::from(
            constants,
            wfs.constants,
            prop_settings.global.take(),
            dao_consts,
            storage,
            Some(global_storage),
        ));

        // Check states
        assert!(
            proposal.state == ProposalState::Accepted,
            "Proposal is not accepted."
        );
        assert!(
            wfi.get_state() == InstanceState::Running,
            "Workflow is not running."
        );
        assert!(activities.get(activity_id).is_some(), "activity not found");

        // Find activity.
        let TemplateActivity {
            is_sync,
            automatic,
            terminal,
            actions,
            postprocessing,
            ..
        } = activities
            .swap_remove(activity_id)
            .into_activity()
            .expect("Activity is init");

        // Loop / other activity case
        if wfi.is_new_transition(activity_id) {
            // Find transition.
            let transition = wfi
                .find_transition(transitions.as_slice(), activity_id)
                .expect("Transition is not possible.");

            // Check transition counter.
            assert!(
                wfi.update_transition_counter(activity_id as usize),
                "Reached transition limit."
            );

            // Check transition condition.
            assert!(
                transition
                    .cond
                    .as_ref()
                    .map(|expr| expr
                        .bind_and_eval(sources.as_ref(), None, expressions.as_slice())
                        .expect("Binding and eval transition condition failed.")
                        .try_into_bool()
                        .expect("Invalid transition condition definition."))
                    .unwrap_or(true),
                "Transition condition failed."
            );
            wfi.register_new_activity(
                activity_id as u8,
                actions.len() as u8,
                terminal == Terminality::Automatic,
            );
        } else {
            assert!(wfi.actions_remaining() > 0, "activity is already finished");
        }

        // Put activity's shared values into Source object if defined.
        if let Some(activity_input) = prop_settings
            .binds
            .get_mut(activity_id)
            .expect("fatal - missing activity bind")
        {
            if let Some(prop_shared) = activity_input.shared.take() {
                sources.set_prop_shared(prop_shared);
            }
        }

        // Create execution context DTO.
        let mut ctx = ActivityContext::new(
            proposal_id,
            activity_id,
            env::predecessor_account_id(),
            env::attached_deposit(),
            prop_settings,
            wfi.actions_done_count(),
            postprocessing,
            terminal,
            actions,
        );

        // Skip rights check for automatic activity.
        // TODO: This might be solved by settings run rights "Anyone" to the automatic activity.
        if automatic {
            // Check rights
            assert!(
                self.check_rights(
                    activity_rights
                        .get(activity_id)
                        .expect("Rights not found in settings.")
                        .as_slice(),
                    &ctx.caller
                ),
                "No rights."
            );
        }

        // Check action input structure including optional actions.
        assert!(
            self.check_activity_input(
                ctx.actions.as_slice(),
                actions_inputs.as_slice(),
                ctx.actions_done_before as usize
            ),
            "Activity input structure is invalid."
        );
        let result;
        if is_sync {
            result = self.run_sync_activity(
                &mut ctx,
                expressions.as_slice(),
                sources.as_mut(),
                actions_inputs,
            );

            // In case not a single DaoAction was executed, then consider this call as failed and panic!
            if result.is_err() || ctx.actions_done() == 0 {
                panic!(
                    "Not a single action was executed. error: {:?}, {:?}",
                    result, ctx
                );
            } else {
                //At least one action was executed.
                // Save storages.
                if let Some(storage) = sources.take_storage() {
                    self.storage.insert(&storage_key.unwrap(), &storage);
                }
                self.storage.insert(
                    &GLOBAL_BUCKET_IDENT.into(),
                    &sources
                        .take_global_storage()
                        .expect("Missing global storage"),
                );
                wfi.new_actions_done(ctx.actions_done(), current_timestamp_sec());
            }
        } else {
            // Optional actions will be immediatelly added to instance.
            let mut optional_actions = 0;
            // In case of fn calls activity storages are mutated in postprocessing as promises resolve.
            result = self.run_async_activity(
                &mut ctx,
                expressions.as_slice(),
                sources.as_mut(),
                actions_inputs,
                &mut optional_actions,
            );
            if result.is_err() || ctx.actions_done() == 0 {
                panic!(
                    "Not a single action was executed. error: {:?}, {:?}",
                    result, ctx
                );
            } else {
                wfi.await_promises(ctx.actions_done());
            }
        }

        // Decide if is fatal error.
        let result = if let Err(e) = result {
            let e = ActivityError::from(e);

            if e.is_fatal() {
                wfi.set_fatal_error();
            }
            Some(e)
        } else {
            None
        };

        if result.is_none() {
            self.register_executed_activity(&ctx.caller, RewardActivity::Activity.into())
        }

        // Save mutated instance state.
        self.workflow_instance.insert(&proposal_id, &wfi);
        result
    }

    // TODO finish error handling
    /// Private callback to check Promise result.
    /// If there's postprocessing, then it's executed.
    /// Postprocessing always requires storage.
    /// Unwrapping is OK as it's been checked before dispatching this promise.
    #[private]
    pub fn postprocess(
        &mut self,
        instance_id: u32,
        action_id: u8,
        must_succeed: bool,
        storage_key: Option<String>,
        postprocessing: Option<Postprocessing>,
    ) {
        assert_eq!(
            env::promise_results_count(),
            1,
            "{}",
            ERR_PROMISE_INVALID_RESULTS_COUNT
        );
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(val) => {
                self.debug_log.push(format!(
                    "promise log: SUCCESS instance_id: {}, action_id: {}; ",
                    instance_id, action_id
                ));
                self.postprocessing_success(
                    instance_id,
                    action_id,
                    storage_key,
                    postprocessing,
                    val,
                )
            }
            PromiseResult::Failed => {
                self.debug_log.push(format!(
                    "promise log: ERROR instance_id: {}, action_id: {}; ",
                    instance_id, action_id
                ));
                self.postprocessing_failed(instance_id, must_succeed)
            }
        }
    }

    /// Changes workflow instance state to finish.
    /// Rights to close are same as the "end" activity rights.
    /// Panic if try to close not accepted proposal.
    pub fn workflow_finish(&mut self, proposal_id: u32) -> bool {
        let caller = env::predecessor_account_id();
        let (proposal, _, wfs) = self.get_workflow_and_proposal(proposal_id);
        assert!(
            proposal.state == ProposalState::Accepted,
            "proposal is not accepted"
        );
        let mut wfi = self.workflow_instance.get(&proposal_id).unwrap();
        // TODO: Transition timestamp should not be included in this case.
        if wfi.get_state() == InstanceState::FatalError
            || self.check_rights(
                wfs.activity_rights[wfi.get_current_activity_id() as usize - 1].as_slice(),
                &caller,
            ) && wfi.try_to_finish()
        {
            self.workflow_instance.insert(&proposal_id, &wfi);
            true
        } else {
            false
        }
    }
}

// Internal action methods.
impl Contract {
    /// Tries to run all activity's actions.
    /// Some checks must be done before calling this function.
    pub fn run_sync_activity(
        &mut self,
        ctx: &mut ActivityContext,
        expressions: &[EExpr],
        sources: &mut dyn Source,
        mut input: Vec<Option<ActionInput>>,
    ) -> Result<(), ActionError> {
        // Loop which tries to execute all actions, starting from the last done. Returns when something goes wrong.
        let last_action_done = ctx.actions_done_before as usize;
        for idx in last_action_done..ctx.actions.len() {
            // Assuming that structure of inputs was checked above therefore unwraping on indexes is OK.
            let mut action_input = match input.get_mut(idx).unwrap().take() {
                Some(a) => a.values.into_activity_input(),
                None => {
                    ctx.set_next_action_done();
                    continue;
                }
            };
            let tpl_action = ctx.actions.get_mut(idx).unwrap();

            // Check exec condition.
            if let Some(cond) = tpl_action.exec_condition.as_ref() {
                if !cond
                    .bind_and_eval(sources, Some(action_input.as_ref()), expressions)?
                    .try_into_bool()?
                {
                    return Err(ActionError::Condition(idx as u8));
                }
            };

            // Assign current action proposal binds to source if there's defined one.
            if let Some(mut binds) = ctx
                .proposal_settings
                .binds
                .get_mut(ctx.activity_id)
                .unwrap()
                .take()
            {
                if let Some(prop_binds) = binds
                    .values
                    .get_mut(idx)
                    .expect("Missing activity bind")
                    .take()
                {
                    sources.set_prop_action(prop_binds);
                }
            } else {
                sources.unset_prop_action();
            }

            // TODO: Refactor.
            let action_data = std::mem::replace(&mut tpl_action.action_data, ActionType::None)
                .try_into_action_data()
                .ok_or(ActionError::InvalidWfStructure(
                    "missing action data".into(),
                ))?;

            // Need metadata coz validations and bindings. Metadata are always included in DAO.
            let binds = action_data.binds.as_slice();

            // Check input validators.
            if !validate(
                sources,
                tpl_action.validators.as_slice(),
                expressions,
                action_input.as_ref(),
            )? {
                return Err(ActionError::Validation(idx as u8));
            }

            // Bind user inputs.
            bind_input(sources, binds, expressions, action_input.as_mut())?;

            let deposit = match &action_data.required_deposit {
                Some(arg_src) => get_value_from_source(sources, arg_src)
                    .map_err(|_| ActionError::InvalidSource)?
                    .try_into_u128()?,
                _ => 0,
            };

            ctx.attached_deposit = ctx
                .attached_deposit
                .checked_sub(deposit)
                .ok_or(ActionError::NotEnoughDeposit)?;

            if action_data.name != DaoActionIdent::Event {
                self.execute_dao_action(ctx.proposal_id, action_data.name, action_input.as_mut())?;
            }

            // TODO: Handle error so we do only part of the batch.
            if let Some(mut pp) = tpl_action.postprocessing.take() {
                pp.bind_instructions(sources, action_input.as_ref())
                    .map_err(|_| ActionError::ActionPostprocessing(idx as u8))?;
                // TODO: Different execute version for DaoActions?
                // TODO: Global storage manipulation.
                let mut storage = sources.take_storage();
                let mut global_storage = sources
                    .take_global_storage()
                    .expect("Global storage must be accessible.");
                if pp
                    .execute(vec![], storage.as_mut(), &mut global_storage, &mut None)
                    .is_err()
                {
                    return Err(ActionError::ActionPostprocessing(idx as u8));
                }
                sources.replace_global_storage(global_storage);
                if let Some(storage) = storage {
                    sources.replace_storage(storage);
                }
            }

            self.debug_log
                .push(format!("dao action executed: {}", ctx.activity_id));
            ctx.set_next_action_done();
        }

        Ok(())
    }
    /// Async version of `run_sync_activity` function.
    pub fn run_async_activity(
        &mut self,
        ctx: &mut ActivityContext,
        expressions: &[EExpr],
        sources: &mut dyn Source,
        mut input: Vec<Option<ActionInput>>,
        optional_actions: &mut u8,
    ) -> Result<(), ActionError> {
        // Loop which tries to execute all actions, starting from the last done. Returns when something goes wrong.
        // Assuming that structure of inputs was checked above therefore unwraping on indexes is OK.
        let last_action_done = ctx.actions_done_before as usize;
        log!("last action done: {}", last_action_done);

        // This strange variable is here because "optional-required-optional" actions case might happen.
        // Therefore we must not considered 3th action as sucessfull but instead of that break the cycle.
        // This might be redundant and "YAGNI" stuff I let it stay here for now.
        let mut optional_state = 0;
        let mut promise: Option<Promise> = None;
        for idx in last_action_done..ctx.actions.len() {
            let mut action_input = match input.get_mut(idx).unwrap().take() {
                Some(a) => {
                    if optional_state == 1 {
                        optional_state = 2;
                    }
                    a.values.into_activity_input()
                }
                None => {
                    if optional_state == 2 {
                        break;
                    }
                    optional_state = 1;
                    ctx.set_next_optional_action_done();
                    ctx.set_next_action_done();
                    continue;
                }
            };

            let tpl_action = ctx.actions.get_mut(idx).unwrap();
            // Check exec condition.
            if let Some(cond) = tpl_action.exec_condition.as_ref() {
                if !cond
                    .bind_and_eval(sources, Some(action_input.as_ref()), expressions)?
                    .try_into_bool()?
                {
                    return Err(ActionError::Condition(idx as u8));
                }
            };

            // Assign current action proposal binds to source if there's defined one.
            if let Some(mut binds) = ctx
                .proposal_settings
                .binds
                .get_mut(ctx.activity_id)
                .unwrap()
                .take()
            {
                if let Some(prop_binds) = binds
                    .values
                    .get_mut(idx)
                    .expect("Missing activity bind")
                    .take()
                {
                    sources.set_prop_action(prop_binds);
                }
            } else {
                sources.unset_prop_action();
            }

            let new_promise;
            if tpl_action.action_data.is_fncall() {
                let action_data = std::mem::replace(&mut tpl_action.action_data, ActionType::None)
                    .try_into_fncall_data()
                    .ok_or(ActionError::InvalidWfStructure(
                        "missing action data".into(),
                    ))?;

                // Metadata are provided by workflow provider when workflow is added.
                // Missing metadata are fault of the workflow provider and are considered as fatal runtime error.
                let (name, method, metadata) =
                    self.get_fncall_id_with_metadata(action_data.id, sources)?;

                if !validate(
                    sources,
                    tpl_action.validators.as_slice(),
                    expressions,
                    action_input.as_ref(),
                )? {
                    return Err(ActionError::Validation(idx as u8));
                }

                let binds = action_data.binds.as_slice();
                bind_input(sources, binds, expressions, action_input.as_mut())?;

                let deposit = match action_data.deposit {
                    Some(arg_src) => get_value_from_source(sources, &arg_src)
                        .map_err(|_| ActionError::InvalidSource)?
                        .try_into_u128()?,
                    None => 0,
                };

                let pp = if let Some(mut pp) = tpl_action.postprocessing.take() {
                    pp.bind_instructions(sources, action_input.as_ref())
                        .map_err(|_| ActionError::ActionPostprocessing(idx as u8))?;
                    Some(pp)
                } else {
                    None
                };
                let args = serialize_to_json(action_input, metadata.as_slice());
                self.debug_log.push(format!(
                    "promise dispatch - contract: {}, method: {}; args: {}; ",
                    &name, &method, &args
                ));
                // Dispatch fncall and its postprocessing.
                new_promise = Promise::new(name)
                    .function_call(
                        method,
                        args.into_bytes(),
                        deposit,
                        Gas(action_data.tgas as u64 * 10u64.pow(12)),
                    )
                    .then(ext_self::postprocess(
                        ctx.proposal_id,
                        idx as u8,
                        tpl_action.must_succeed,
                        ctx.proposal_settings.storage_key.clone(),
                        pp,
                        env::current_account_id(),
                        0,
                        Gas(50 * 10u64.pow(12)),
                    ));
            } else {
                let (sender_src, amount_src) =
                    std::mem::replace(&mut tpl_action.action_data, ActionType::None)
                        .try_into_send_near_sources()
                        .ok_or(ActionError::InvalidWfStructure(
                            "send near invalid data".into(),
                        ))?;
                let name = match &sender_src {
                    User(key) => action_input
                        .get(&key)
                        .ok_or(ActionError::InputStructure(idx as u8))?
                        .to_owned(),
                    _ => get_value_from_source(sources, &sender_src)
                        .map_err(|_| ActionError::InputStructure(idx as u8))?,
                }
                .try_into_string()?;

                let amount = match &amount_src {
                    User(key) => action_input
                        .get(&key)
                        .ok_or(ActionError::InputStructure(idx as u8))?
                        .to_owned(),
                    _ => get_value_from_source(sources, &amount_src)
                        .map_err(|_| ActionError::InputStructure(idx as u8))?,
                }
                .try_into_u128()?;
                self.debug_log.push(format!(
                    "promise send near - name: {}, amount: {}; ",
                    &name, amount,
                ));
                new_promise =
                    Promise::new(AccountId::try_from(name).expect("invalid account_id name"))
                        .transfer(amount)
                        .then(ext_self::postprocess(
                            ctx.proposal_id,
                            idx as u8,
                            tpl_action.must_succeed,
                            ctx.proposal_settings.storage_key.clone(),
                            None,
                            env::current_account_id(),
                            0,
                            Gas(10 * 10u64.pow(12)),
                        ));
            }

            promise = if let Some(p) = promise {
                Some(p.and(new_promise))
            } else {
                Some(new_promise)
            };

            // Number of successfully dispatched promises.
            ctx.set_next_action_done();
        }

        Ok(())
    }

    // TODO: Review process.
    /// Error callback.
    /// If promise did not have to succeed, then instance is still updated.
    pub fn postprocessing_failed(&mut self, proposal_id: u32, must_succeed: bool) {
        let mut wfi = self.workflow_instance.get(&proposal_id).unwrap();
        if must_succeed {
            wfi.promise_failed();
        } else {
            let timestamp = current_timestamp_sec();
            wfi.promise_success();
            //wfi.try_to_advance_activity();
            wfi.new_actions_done(1, timestamp);
        }
        self.workflow_instance.insert(&proposal_id, &wfi);
    }

    // TODO: Review process.
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
        let mut wfi = self.workflow_instance.get(&proposal_id).unwrap();
        // Action transaction check if previous action succesfully finished.
        if wfi.check_invalid_action(action_id) {
            self.workflow_instance.insert(&proposal_id, &wfi);
            return;
        }
        // Check if its first action done in the activity
        wfi.promise_success();
        wfi.new_actions_done(1, current_timestamp_sec());
        log!("wfi after update: {:?}", wfi);

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
                wfi.set_fatal_error();
            } else {
                // Only in case its workflow Add.
                if let Some((workflow, fncalls, fncall_metadata)) = new_template {
                    // Unwraping is ok as settings are inserted when this proposal is accepted.
                    let settings = self
                        .proposed_workflow_settings
                        .remove(&proposal_id)
                        .unwrap();

                    self.workflow_last_id += 1;
                    self.workflow_template
                        .insert(&self.workflow_last_id, &(workflow, settings));
                    self.init_function_calls(fncalls, fncall_metadata);
                }
                // Save updated storages.
                if let Some(storage) = storage {
                    self.storage.insert(&storage_key.unwrap(), &storage);
                }
                self.storage
                    .insert(&GLOBAL_BUCKET_IDENT.into(), &global_storage);
            }
        };
        log!("wfi before save: {:?}", wfi);
        self.workflow_instance.insert(&proposal_id, &wfi);
    }

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
                    Some(group) => {
                        if group.is_member(account_id) {
                            return true;
                        } else {
                            continue;
                        }
                    }
                    _ => continue,
                },
                ActivityRight::GroupMember(g, name) => {
                    if *name != *account_id {
                        continue;
                    }
                    match self.groups.get(g) {
                        Some(group) => match group.is_member(account_id) {
                            true => return true,
                            false => continue,
                        },
                        _ => continue,
                    }
                }
                ActivityRight::TokenHolder => {
                    if self.delegations.get(account_id).unwrap_or(0) > 0 {
                        return true;
                    } else {
                        continue;
                    }
                }
                ActivityRight::GroupRole(g, r) => match self.user_roles.get(account_id) {
                    Some(roles) => {
                        if roles.has_group_role(*g, *r) {
                            return true;
                        } else {
                            continue;
                        }
                    }
                    _ => continue,
                },
                ActivityRight::GroupLeader(g) => match self.groups.get(g) {
                    Some(group) => {
                        if group.is_account_id_leader(account_id) {
                            return true;
                        } else {
                            continue;
                        }
                    }
                    _ => continue,
                },
                ActivityRight::Member => {
                    if self.user_roles.get(&account_id).is_some() {
                        return true;
                    } else {
                        continue;
                    }
                }
                ActivityRight::Account(a) => match *a == *account_id {
                    true => return true,
                    false => continue,
                },
            }
        }
        false
    }

    /// Checks if inputs structure is same as activity definition.
    /// First action input must belong to next action to be done.
    /// Same order as activity's actions is required.
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
            DaoActionIdent::TreasuryAddPartition => {
                let partition = try_bind_partition(inputs).expect("failed to bind partition");
                self.partition_add(partition);
            }
            DaoActionIdent::RewardAdd => {
                let reward = try_bind_reward(inputs).expect("failed to bind reward");
                self.reward_add(reward);
            }
            DaoActionIdent::GroupAdd => {
                let group = try_bind_group(inputs);
                self.group_add(group);
            }
            DaoActionIdent::GroupRemove => {
                let id = inputs
                    .get("id")
                    .expect("missing group id")
                    .try_into_u64()
                    .expect("invalid datatype: group id") as u16;
                self.group_remove(id);
            }
            DaoActionIdent::GroupAddMembers => {
                let id = inputs
                    .get("id")
                    .expect("missing group id")
                    .try_into_u64()
                    .expect("invalid datatype: group id") as u16;
                let members = try_bind_group_members("members", inputs);
                let member_roles = try_to_bind_member_roles("member_roles", inputs);
                self.group_add_members(id, members, member_roles);
            }
            DaoActionIdent::GroupRemoveMembers => {
                let id = inputs
                    .get("id")
                    .expect("missing group id")
                    .try_into_u64()
                    .expect("invalid datatype: group id") as u16;

                let members = try_bind_accounts("members", inputs);
                if !members.is_empty() {
                    self.group_remove_members(id, members);
                }
            }
            DaoActionIdent::GroupRemoveRoles => {
                let id = inputs
                    .get("id")
                    .expect("missing group id")
                    .try_into_u64()
                    .expect("invalid datatype: group id") as u16;

                let roles = try_to_bind_roles("role_ids", inputs);
                self.group_remove_roles(id, roles);
            }
            DaoActionIdent::GroupRemoveMemberRoles => {
                let id = inputs
                    .get("id")
                    .expect("missing group id")
                    .try_into_u64()
                    .expect("invalid datatype: group id") as u16;
                let member_roles = try_to_bind_member_roles("member_roles", inputs);
                self.group_remove_member_roles(id, member_roles);
            }
            DaoActionIdent::TagAdd => {
                todo!();
            }
            _ => unreachable!(),
        }

        Ok(())
    }

    // TODO: Tests.
    /// Binds dao FnCall
    pub fn get_fncall_id_with_metadata(
        &self,
        id: FnCallIdType,
        sources: &dyn Source,
    ) -> Result<(AccountId, MethodName, Vec<ObjectMetadata>), ActionError> {
        let data = match id {
            FnCallIdType::Static(account, method) => (
                account.clone(),
                method.clone(),
                self.function_call_metadata
                    .get(&(account, method.clone()))
                    .ok_or(ActionError::MissingFnCallMetadata(method))?,
            ),
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
            FnCallIdType::StandardStatic(account, method) => (
                account.clone(),
                method.clone(),
                self.standard_function_call_metadata
                    .get(&method.clone())
                    .ok_or(ActionError::MissingFnCallMetadata(method))?,
            ),
            FnCallIdType::StandardDynamic(arg_src, method) => {
                let name = get_value_from_source(sources, &arg_src)
                    .map_err(ProcessingError::Source)?
                    .try_into_string()?;
                (
                    AccountId::try_from(name.to_string())
                        .map_err(|_| ActionError::InvalidDataType)?,
                    method.clone(),
                    self.standard_function_call_metadata
                        .get(&method)
                        .ok_or(ActionError::MissingFnCallMetadata(method))?,
                )
            }
        };
        Ok(data)
    }
}
