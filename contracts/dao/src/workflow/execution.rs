use library::functions::binding::bind_input;
use library::functions::evaluation::eval;
use library::functions::serialization::serialize_to_json;
use library::functions::validation::validate;
use library::interpreter::expression::EExpr;
use library::MethodName;

use library::storage::StorageBucket;
use library::workflow::{
    action::{
        ActionData, ActionInput, FnCallIdType,
        InputSource::{PropSettings, User},
        TemplateAction,
    },
    activity::{TemplateActivity, Terminality},
    instance::InstanceState,
    postprocessing::Postprocessing,
    runtime::activity_input::ActivityInput,
    runtime::source::{DefaultSource, Source},
    settings::TemplateSettings,
    template::Template,
    types::{ActivityRight, DaoActionIdent, ObjectMetadata},
};
use near_sdk::env::panic_str;
use near_sdk::{
    env, ext_contract, log, near_bindgen, require, AccountId, Gas, Promise, PromiseResult,
};

use super::deserialize::{
    deser_account_ids, deser_group_input, deser_group_members, deser_media, deser_member_roles,
    deser_partition, deser_reward, deser_roles_ids, deser_u128, deser_u64,
};
use super::error::{ActionError, ActivityError};
use crate::constants::GLOBAL_BUCKET_IDENT;
use crate::contract::*;
use crate::internal::utils::current_timestamp_sec;
use crate::internal::ActivityContext;
use crate::proposal::ProposalState;
use crate::reward::RewardActivity;
use crate::treasury::AssetRegistrar;

#[ext_contract(ext_self)]
trait CbActivity {
    fn postprocess(
        &mut self,
        instance_id: u32,
        action_id: u8,
        must_succeed: bool,
        storage_key: Option<String>,
        postprocessing: Option<Postprocessing>,
    );
}

#[near_bindgen]
impl Contract {
    // TODO: Auto-finish WF then there is no other possible transition regardless terminality.
    /// Workflow execution entry-point function.
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
        let runtime_constants = self.runtime_constants();

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
            prop_settings.constants.take(),
            runtime_constants,
            storage,
            Some(global_storage),
        ));

        // Check states.
        require!(
            proposal.state == ProposalState::Accepted,
            "Proposal is not accepted."
        );
        require!(
            wfi.get_state() == InstanceState::Running,
            "Workflow is not running."
        );
        require!(activities.get(activity_id).is_some(), "activity not found");

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

        if wfi.is_current_activity_finished() {
            // Find transition.
            let transition = wfi
                .find_transition(transitions.as_slice(), activity_id)
                .expect("Transition is not possible.");

            // Check transition counter.
            require!(
                wfi.update_transition_counter(activity_id as usize),
                "Reached transition limit."
            );

            // Check transition condition.
            require!(
                transition
                    .cond
                    .as_ref()
                    .map(
                        |src| eval(src, sources.as_mut(), expressions.as_slice(), None)
                            .expect("Binding and eval transition condition failed.")
                            .try_into_bool()
                            .expect("Invalid transition condition definition.")
                    )
                    .unwrap_or(true),
                "Transition condition failed."
            );
            wfi.register_new_activity(
                activity_id as u8,
                actions.len() as u8,
                terminal == Terminality::Automatic,
            );
        } else {
            require!(
                activity_id as u8 == wfi.get_current_activity_id(),
                "Current activity must be finished first."
            );
        }

        // Put activity's shared values into Source object if defined.
        if let Some(activity_input) = prop_settings
            .activity_constants
            .get_mut(activity_id)
            .expect("fatal - missing activity bind")
        {
            if let Some(prop_shared) = activity_input.constants.take() {
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
        if automatic {
            // Check rights
            require!(
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
        require!(
            self.check_activity_input(
                ctx.actions.as_slice(),
                actions_inputs.as_slice(),
                ctx.actions_done_before as usize
            ),
            "Activity input structure is invalid."
        );
        let result = if is_sync {
            // Sync might panic.
            self.run_sync_activity(
                &mut ctx,
                expressions.as_slice(),
                sources.as_mut(),
                actions_inputs,
            );
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
            None
        } else {
            // Async cannot panic immediately.
            let result = self.run_async_activity(
                &mut ctx,
                expressions.as_slice(),
                sources.as_mut(),
                actions_inputs,
            );
            // Zero action executed means no promise was dispatched.
            if result.is_err() && ctx.actions_done() == 0 {
                panic!("No action was executed. error: {:?}, {:?}", result, ctx);
            } else {
                wfi.await_promises(
                    ctx.optional_actions_done(),
                    ctx.actions_done() - ctx.optional_actions_done(),
                );
            }
            if let Err(e) = result {
                let e = ActivityError::from(e);
                if e.is_fatal() {
                    wfi.set_fatal_error();
                    log!("wf fatal error: {:?}", e);
                }
                Some(e)
            } else {
                None
            }
        };
        self.register_executed_activity(&ctx.caller, RewardActivity::Activity.into());
        self.workflow_instance.insert(&proposal_id, &wfi);
        result
    }

    /// Private callback to check promise result.
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
        require!(
            env::promise_results_count() == 1,
            "invalid promise result count"
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

    /// Changes workflow instance state to finished.
    /// Rights to finish workflow are same as the "end" activity rights.
    /// Panic if proposal is not accepted.
    pub fn workflow_finish(&mut self, proposal_id: u32) -> bool {
        let caller = env::predecessor_account_id();
        let (proposal, _, wfs) = self.get_workflow_and_proposal(proposal_id);
        require!(
            proposal.state == ProposalState::Accepted,
            "proposal is not accepted"
        );
        let mut wfi = self.workflow_instance.get(&proposal_id).unwrap();
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

impl Contract {
    /// Tries to run all activity's actions.
    /// Some checks must be done before calling this function.
    /// Panics if anything goes wrong.
    /// Panicking is used in order to reduce gas/storage fees.
    pub fn run_sync_activity(
        &mut self,
        ctx: &mut ActivityContext,
        expressions: &[EExpr],
        sources: &mut dyn Source,
        mut input: Vec<Option<ActionInput>>,
    ) {
        // Loop which tries to execute all actions, starting from the last done. Returns when something goes wrong.
        let last_action_done = ctx.actions_done_before as usize;
        for idx in 0..input.len() {
            // Assuming that structure of inputs was checked above therefore unwraping on indexes is OK.
            set_action_propose_binds(idx + last_action_done, sources, ctx).unwrap();
            let tpl_action = ctx.actions.get_mut(idx + last_action_done).unwrap();
            let mut action_input = match tpl_action.input_source {
                User => match input.get_mut(idx).unwrap().take() {
                    Some(a) => a.values.into_activity_input(),
                    None => {
                        ctx.set_next_action_done();
                        continue;
                    }
                },
                PropSettings => sources
                    .unset_prop_action()
                    .expect("missing action inputs")
                    .into_activity_input(),
            };
            // Check exec condition.
            if let Some(cond) = tpl_action.exec_condition.as_ref() {
                if !eval(cond, sources, expressions, Some(action_input.as_ref()))
                    .unwrap()
                    .try_into_bool()
                    .unwrap()
                {
                    panic_str("exec condition was not met")
                }
            };

            // Assign current action proposal binds to source if there's defined one.
            let user_inputs = action_input.to_vec();
            let action_data = std::mem::replace(&mut tpl_action.action_data, ActionData::None)
                .try_into_action_data()
                .expect("missing action data");

            // Check input validators.
            if !validate(
                sources,
                tpl_action.validators.as_slice(),
                expressions,
                action_input.as_ref(),
            )
            .unwrap()
            {
                panic_str("validation failed")
            }
            // Bind user inputs.
            let binds = action_data.binds.as_slice();
            bind_input(sources, binds, expressions, action_input.as_mut()).unwrap();

            let deposit = match &action_data.required_deposit {
                Some(arg_src) => eval(arg_src, sources, expressions, None)
                    .unwrap()
                    .try_into_u128()
                    .unwrap(),
                _ => 0,
            };
            ctx.attached_deposit = ctx
                .attached_deposit
                .checked_sub(deposit)
                .expect("not enough attached deposit");
            if action_data.name != DaoActionIdent::Event {
                self.execute_dao_action(action_data.name, action_input.as_mut())
                    .unwrap();
            }
            if let Some(mut pp) = tpl_action.postprocessing.take() {
                pp.bind_instructions(sources, expressions, action_input.as_ref())
                    .unwrap();
                let mut storage = sources.take_storage();
                let mut global_storage = sources.take_global_storage().unwrap();
                pp.execute(vec![], storage.as_mut(), &mut global_storage, &mut None)
                    .unwrap();
                sources.replace_global_storage(global_storage);
                if let Some(storage) = storage {
                    sources.replace_storage(storage);
                }
            }
            self.debug_log
                .push(format!("dao action executed: {}", ctx.activity_id));
            self.log_action(
                ctx.proposal_id,
                env::predecessor_account_id(),
                ctx.activity_id as u8,
                idx as u8,
                user_inputs,
            );
            ctx.set_next_action_done();
        }
    }
    /// Async version of `run_sync_activity` function with one difference.
    /// Does not panic but propagate Err to the caller.
    /// This is important as any promise could already been dispatched.
    pub fn run_async_activity(
        &mut self,
        ctx: &mut ActivityContext,
        expressions: &[EExpr],
        sources: &mut dyn Source,
        mut input: Vec<Option<ActionInput>>,
    ) -> Result<(), ActionError> {
        // Loop which tries to execute all actions, starting from the last done. Returns when something goes wrong.
        // Assuming that structure of inputs was checked above therefore unwraping on indexes is OK.
        let last_action_done = ctx.actions_done_before as usize;
        log!("last action done: {}", last_action_done);
        let mut required_promise_dispatched = false;
        let mut promise: Option<Promise> = None;
        for idx in 0..input.len() {
            // Assuming that structure of inputs was checked above therefore unwraping on indexes is OK.
            set_action_propose_binds(idx + last_action_done, sources, ctx)?;
            let tpl_action = ctx.actions.get_mut(idx + last_action_done).unwrap();
            let mut action_input = match tpl_action.input_source {
                User => match input.get_mut(idx).unwrap().take() {
                    Some(a) => {
                        if !tpl_action.optional {
                            required_promise_dispatched = true;
                        }
                        a.values.into_activity_input()
                    }
                    None => {
                        if required_promise_dispatched {
                            break;
                        }
                        ctx.set_next_optional_action_done();
                        ctx.set_next_action_done();
                        continue;
                    }
                },
                PropSettings => sources
                    .unset_prop_action()
                    .ok_or_else(|| ActionError::InvalidWfStructure("missing action inputs".into()))?
                    .into_activity_input(),
            };
            // Check exec condition.
            if let Some(cond) = tpl_action.exec_condition.as_ref() {
                if !eval(cond, sources, expressions, Some(action_input.as_ref()))?
                    .try_into_bool()?
                {
                    return Err(ActionError::Condition(idx as u8));
                }
            };
            let user_inputs = action_input.to_vec();
            let new_promise = if tpl_action.action_data.is_fncall() {
                let action_data = std::mem::replace(&mut tpl_action.action_data, ActionData::None)
                    .try_into_fncall_data()
                    .ok_or_else(|| {
                        ActionError::InvalidWfStructure("missing fncall action data".into())
                    })?;

                let (name, method, metadata) = self.load_fncall_id_with_metadata(
                    action_data.id,
                    sources,
                    expressions,
                    action_input.as_ref(),
                )?;

                if !validate(
                    sources,
                    tpl_action.validators.as_slice(),
                    expressions,
                    action_input.as_ref(),
                )? {
                    return Err(ActionError::Validation);
                }

                let binds = action_data.binds.as_slice();
                bind_input(sources, binds, expressions, action_input.as_mut())?;

                let deposit = match action_data.deposit {
                    Some(arg_src) => eval(&arg_src, sources, expressions, None)?.try_into_u128()?,
                    None => 0,
                };

                let pp = if let Some(mut pp) = tpl_action.postprocessing.take() {
                    pp.bind_instructions(sources, expressions, action_input.as_ref())?;
                    Some(pp)
                } else {
                    None
                };
                let args = serialize_to_json(action_input, metadata.as_slice())?;
                self.debug_log.push(format!(
                    "promise dispatch - contract: {}, method: {}; args: {}; ",
                    &name, &method, &args
                ));
                // Dispatch fncall and its postprocessing.
                Promise::new(name)
                    .function_call(
                        method,
                        args.into_bytes(),
                        deposit,
                        Gas(action_data.tgas as u64 * 10u64.pow(12)),
                    )
                    .then(
                        ext_self::ext(env::current_account_id())
                            .with_static_gas(Gas(50 * 10u64.pow(12)))
                            .postprocess(
                                ctx.proposal_id,
                                idx as u8,
                                action_data.must_succeed,
                                ctx.proposal_settings.storage_key.clone(),
                                pp,
                            ),
                    )
            } else {
                let (sender_src, amount_src) =
                    std::mem::replace(&mut tpl_action.action_data, ActionData::None)
                        .try_into_send_near_sources()
                        .ok_or_else(|| {
                            ActionError::InvalidWfStructure("expected send near data".into())
                        })?;
                let name = eval(
                    &sender_src,
                    sources,
                    expressions,
                    Some(action_input.as_ref()),
                )?
                .try_into_string()?;
                let amount = eval(
                    &amount_src,
                    sources,
                    expressions,
                    Some(action_input.as_ref()),
                )?
                .try_into_u128()?;
                self.debug_log.push(format!(
                    "promise send near - name: {}, amount: {}; ",
                    &name, amount,
                ));
                let pp = if let Some(mut pp) = tpl_action.postprocessing.take() {
                    pp.bind_instructions(sources, expressions, action_input.as_ref())?;
                    Some(pp)
                } else {
                    None
                };
                Promise::new(
                    AccountId::try_from(name)
                        .map_err(|_| ActionError::ParseAccountId(sender_src.is_user_input()))?,
                )
                .transfer(amount)
                .then(
                    ext_self::ext(env::current_account_id())
                        .with_static_gas(Gas(10 * 10u64.pow(12)))
                        .postprocess(
                            ctx.proposal_id,
                            idx as u8,
                            true,
                            ctx.proposal_settings.storage_key.clone(),
                            pp,
                        ),
                )
            };
            promise = if let Some(p) = promise {
                Some(p.and(new_promise))
            } else {
                Some(new_promise)
            };
            // Number of successfully dispatched promises.
            self.log_action(
                ctx.proposal_id,
                env::predecessor_account_id(),
                ctx.activity_id as u8,
                idx as u8,
                user_inputs,
            );
            ctx.set_next_action_done();
        }
        Ok(())
    }

    /// Error callback.
    /// If promise did not have to succeed, then instance is still updated.
    pub fn postprocessing_failed(&mut self, proposal_id: u32, must_succeed: bool) {
        let mut wfi = self.workflow_instance.get(&proposal_id).unwrap();
        if must_succeed {
            wfi.promise_failed();
        } else {
            let timestamp = current_timestamp_sec();
            wfi.promise_success();
            wfi.new_actions_done(1, timestamp);
        }
        self.workflow_instance.insert(&proposal_id, &wfi);
    }

    /// Success callback.
    /// Update workflow's instance.
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
        if wfi.check_invalid_action(action_id) {
            self.workflow_instance.insert(&proposal_id, &wfi);
            return;
        }

        // Execute postprocessing script which must always succeed - fatal error otherwise.
        if let Some(pp) = postprocessing {
            let mut global_storage = self.storage.get(&GLOBAL_BUCKET_IDENT.into()).unwrap();
            let mut storage = if let Some(ref storage_key) = storage_key {
                self.storage.get(storage_key)
            } else {
                None
            };
            let mut new_template = None;
            let result = pp.execute(
                promise_call_result,
                storage.as_mut(),
                &mut global_storage,
                &mut new_template,
            );
            if result.is_err() {
                wfi.set_fatal_error();
                log!("WF FATAL ERROR: {:?}", result);
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
        wfi.promise_success();
        wfi.new_actions_done(1, current_timestamp_sec());
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
                    if self.user_roles.get(account_id).is_some() {
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
        require!(
            !inputs.is_empty() && inputs.len() + actions_done <= actions.len(),
            "Action input has invalid length."
        );
        for (idx, action) in inputs.iter().enumerate() {
            let template_action = actions.get(idx + actions_done).unwrap();
            match (template_action.optional, action) {
                (_, Some(a)) => {
                    if !a.action.eq(&template_action.action_data) {
                        return false;
                    }
                }
                (false, None) => return false,
                _ => continue,
            }
        }
        true
    }

    /// Executes DAO's native action.
    pub fn execute_dao_action(
        &mut self,
        action_ident: DaoActionIdent,
        inputs: &mut dyn ActivityInput,
    ) -> Result<(), ActionError> {
        match action_ident {
            DaoActionIdent::TreasuryAddPartition => {
                let partition = deser_partition(inputs, self as &mut dyn AssetRegistrar)?;
                self.partition_add(partition);
            }
            DaoActionIdent::PartitionAddAssetAmount => {
                let id = deser_u64("id", inputs)? as u16;
                let asset_id = deser_u64("asset_id", inputs)? as u8;
                let amount = deser_u128("amount", inputs)?;
                self.partition_add_asset_amount(id, asset_id, amount);
            }
            DaoActionIdent::RewardAdd => {
                let reward = deser_reward(inputs)?;
                self.reward_add(reward)?;
            }
            DaoActionIdent::RewardUpdate => {
                let id = deser_u64("id", inputs)? as u16;
                let time_valid_to = deser_u64("time_valid_to", inputs)? as u64;
                self.reward_update(id, time_valid_to)?;
            }
            DaoActionIdent::GroupAdd => {
                let group = deser_group_input(inputs)?;
                self.group_add(group);
            }
            DaoActionIdent::GroupRemove => {
                let id = deser_u64("id", inputs)? as u16;
                self.group_remove(id);
            }
            DaoActionIdent::GroupAddMembers => {
                let id = deser_u64("id", inputs)? as u16;
                let members = deser_group_members("members", inputs)?;
                let member_roles = deser_member_roles("member_roles", inputs)?;
                self.group_add_members(id, members, member_roles);
            }
            DaoActionIdent::GroupRemoveMembers => {
                let id = deser_u64("id", inputs)? as u16;
                let members = deser_account_ids("members", inputs)?;
                self.group_remove_members(id, members);
            }
            DaoActionIdent::GroupRemoveRoles => {
                let id = deser_u64("id", inputs)? as u16;
                let roles = deser_roles_ids("role_ids", inputs)?;
                self.group_remove_roles(id, roles);
            }
            DaoActionIdent::GroupRemoveMemberRoles => {
                let id = deser_u64("id", inputs)? as u16;
                let member_roles = deser_member_roles("member_roles", inputs)?;
                self.group_remove_member_roles(id, member_roles);
            }
            DaoActionIdent::MediaAdd => {
                let media = deser_media("", inputs)?;
                self.media_add(&media);
            }
            DaoActionIdent::MediaUpdate => {
                let id = deser_u64("id", inputs)? as u32;
                let media = deser_media("media", inputs)?;
                self.media_update(id, &media);
            }
            DaoActionIdent::TagAdd => {
                todo!();
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    /// Return function call receiver and method name with all necessary object metadata.
    pub fn load_fncall_id_with_metadata(
        &self,
        id: FnCallIdType,
        sources: &dyn Source,
        expressions: &[EExpr],
        inputs: &dyn ActivityInput,
    ) -> Result<(AccountId, MethodName, Vec<ObjectMetadata>), ActionError> {
        let data = match id {
            FnCallIdType::Static(account, method) => (
                account.clone(),
                method.clone(),
                self.function_call_metadata
                    .get(&(account, method))
                    .ok_or_else(|| {
                        ActionError::InvalidWfStructure("missing fn call metadata".into())
                    })?,
            ),
            FnCallIdType::Dynamic(arg_src, method) => {
                let name = eval(&arg_src, sources, expressions, Some(inputs))?.try_into_string()?;
                (
                    AccountId::try_from(name.to_string())
                        .map_err(|_| ActionError::ParseAccountId(arg_src.is_user_input()))?,
                    method.clone(),
                    self.function_call_metadata
                        .get(&(
                            AccountId::try_from(name).map_err(|_| {
                                ActionError::ParseAccountId(arg_src.is_user_input())
                            })?,
                            method,
                        ))
                        .ok_or_else(|| {
                            ActionError::InvalidWfStructure("missing fn call metadata".into())
                        })?,
                )
            }
            FnCallIdType::StandardStatic(account, method) => (
                account,
                method.clone(),
                self.standard_function_call_metadata
                    .get(&method)
                    .ok_or_else(|| {
                        ActionError::InvalidWfStructure("missing standard fn call metadata".into())
                    })?,
            ),
            FnCallIdType::StandardDynamic(arg_src, method) => {
                let name = eval(&arg_src, sources, expressions, Some(inputs))?.try_into_string()?;
                (
                    AccountId::try_from(name)
                        .map_err(|_| ActionError::ParseAccountId(arg_src.is_user_input()))?,
                    method.clone(),
                    self.standard_function_call_metadata
                        .get(&method)
                        .ok_or_else(|| {
                            ActionError::InvalidWfStructure(
                                "missing standard fn call metadata".into(),
                            )
                        })?,
                )
            }
        };
        Ok(data)
    }
}

/// Set action propose binds to the sources if exist.
/// Unset previous.
/// NOTE: Unwrapping is OK as these values are checked
/// when proposal is being created.
fn set_action_propose_binds(
    action_id: usize,
    sources: &mut dyn Source,
    ctx: &mut ActivityContext,
) -> Result<(), ActionError> {
    if let Some(mut binds) = ctx
        .proposal_settings
        .activity_constants
        .get_mut(ctx.activity_id)
        .unwrap()
        .take()
    {
        if let Some(prop_binds) = binds
            .actions_constants
            .get_mut(action_id)
            .ok_or_else(|| ActionError::InvalidWfStructure("missing activity propose bind".into()))?
            .take()
        {
            sources.set_prop_action(prop_binds);
        }
    } else {
        sources.unset_prop_action();
    }
    Ok(())
}
