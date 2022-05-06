use library::functions::serialization::serialize_to_json;
use library::functions::validation::validate;
use library::functions::{binding::bind_input, utils::get_value_from_source};
use library::interpreter::expression::EExpr;

use library::storage::StorageBucket;
use library::types::datatype::Value;
use library::types::source::{DefaultSource, Source};
use library::workflow::action::{ActionData, ActionInput};
use library::workflow::activity::{TemplateActivity, Terminality};
use library::workflow::instance::InstanceState;
use library::workflow::settings::TemplateSettings;
use library::workflow::template::Template;
use near_sdk::{env, log, near_bindgen, AccountId, Gas, Promise};

use crate::callback::ext_self;
use crate::constants::{EVENT_CALLER_KEY, GLOBAL_BUCKET_IDENT};
use crate::error::{ActionError, ActivityError};
use crate::group::{GroupInput, GroupMember, GroupSettings};
use crate::internal::utils::current_timestamp_sec;
use crate::internal::ActivityContext;
use crate::proposal::ProposalState;
use crate::settings::{assert_valid_dao_settings, DaoSettings};
use crate::tags::Tags;
use crate::token_lock::TokenLock;
use crate::{core::*, GroupId, TagId};

#[near_bindgen]
impl Contract {
    // TODO: Auto-finish WF then there is no other possible transition regardless terminality.
    #[payable]
    #[handle_result]
    pub fn workflow_run_activity(
        &mut self,
        proposal_id: u32,
        activity_id: usize,
        actions_inputs: Vec<Option<ActionInput>>,
    ) -> Result<(), ActivityError> {
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

        // Finds activity.
        let TemplateActivity {
            is_dao_activity,
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
            // Find transition
            let transition = wfi
                .find_transition(transitions.as_slice(), activity_id)
                .expect("Transition is not possible.");

            // Check transition counter
            assert!(
                wfi.update_transition_counter(activity_id as usize),
                "Reached transition limit."
            );

            // Check transition condition
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
        //dbg!(actions_inputs.clone());
        //assert!(
        assert!(
            self.check_activity_input(
                ctx.actions.as_slice(),
                actions_inputs.as_slice(),
                ctx.actions_done_before as usize
            ),
            "Activity input structure is invalid."
        );
        let result;
        if is_dao_activity {
            result = self.run_dao_activity(
                &mut ctx,
                expressions.as_slice(),
                sources.as_mut(),
                actions_inputs,
            );

            // In case not a single DaoAction was executed, then consider this call as failed and panic!
            if result.is_err() || ctx.actions_done() == 0 {
                panic!("Not a single action was executed.");
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
            result = self.run_fncall_activity(
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
        let result: Result<(), ActivityError> = if let Err(e) = result {
            let e = ActivityError::from(e);

            if e.is_fatal() {
                wfi.set_fatal_error();
            }
            Err(e)
        } else {
            Ok(())
        };

        // Save mutated instance state.
        self.workflow_instance.insert(&proposal_id, &wfi);

        result
    }
}

// Internal action methods.
impl Contract {
    /// Tries to run all activity's actions.
    /// Some checks must be done before calling this function.
    pub fn run_dao_activity(
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

            let action_data = std::mem::replace(&mut tpl_action.action_data, ActionData::None)
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

            if action_data.is_event() {
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

                // Insert caller into 0th position.
                action_input.set(EVENT_CALLER_KEY, Value::String(ctx.caller.to_string()));
            } else {
                self.execute_dao_action(ctx.proposal_id, action_data.name, action_input.as_mut())?;
            }

            // TODO: Handle error so we do only part of the batch.
            if let Some(mut pp) = tpl_action.postprocessing.take() {
                pp.bind_instructions(sources, action_input.as_ref())
                    .map_err(|_| ActionError::ActionPostprocessing(idx as u8))?;
                // TODO: Different execute version for DaoActions?
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
            }

            ctx.set_next_action_done();
        }

        Ok(())
    }
    /// FnCall version of `run_dao_activity` function.
    pub fn run_fncall_activity(
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

            let action_data = std::mem::replace(&mut tpl_action.action_data, ActionData::None)
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
            let new_promise = Promise::new(name)
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
