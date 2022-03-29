use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
};

use crate::TransitionLimit;

use super::{activity::Transition, template::Template};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone))]
#[serde(crate = "near_sdk::serde")]
pub enum InstanceState {
    /// Waiting for proposal to be accepted.
    Waiting,
    /// Workflow is running.
    Running,
    /// Unrecoverable error happened. Eg. by executing badly defined workflow.
    FatalError,
    /// Workflow was finished and closed.
    Finished,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Instance {
    pub state: InstanceState,
    pub last_transition_done_at: u64,
    pub current_activity_id: u8,
    pub previous_activity_id: u8,
    pub transition_counter: Vec<Vec<u16>>,
    /// Last activity's count of done actions. < actions.len() during execution.
    pub actions_done_count: u8,
    pub actions_total: u8,
    pub template_id: u16,
}

impl Instance {
    pub fn new(template_id: u16) -> Self {
        Instance {
            state: InstanceState::Waiting,
            last_transition_done_at: 0,
            current_activity_id: 0,
            previous_activity_id: 0,
            actions_done_count: 0,
            actions_total: 0,
            transition_counter: Vec::default(),
            template_id,
        }
    }

    pub fn init_transition_counter(&mut self, counter: Vec<Vec<u16>>) {
        self.transition_counter = counter;
    }

    pub fn check_transition_counter(
        &self,
        activity_id: usize,
        transition_limits: &[Vec<TransitionLimit>],
    ) -> bool {
        *self.transition_counter[self.current_activity_id as usize]
            .get(activity_id)
            .expect("Transition does not exists.")
            < transition_limits[self.current_activity_id as usize][activity_id as usize]
    }

    pub fn find_transition<'a>(
        &self,
        template: &'a Template,
        activity_id: usize,
    ) -> Option<&'a Transition> {
        // Current activity is not finished yet.
        if self.actions_done_count != self.actions_total {
            return None;
        }

        template
            .transitions
            .get(self.current_activity_id as usize)
            .expect("Activity does not exists.")
            .iter()
            .find(|t| t.activity_id == activity_id as u8)
    }

    /*
    // TODO optimalize so we dont have to subtract index by one each time
    /// Finds transition for dao action
    pub fn get_target_trans_with_for_dao_action(
        &self,
        wft: &Template,
        action_ident: ActionType,
    ) -> Option<(TransitionId, ActivityId)> {
        wft.transitions
            .get(self.current_activity_id as usize)
            .map(|t| {
                t.iter()
                    .enumerate()
                    .find(|(_, act_id)| {
                        wft.activities[**act_id as usize].as_ref().unwrap().action == action_ident
                    })
                    .map(|(t_id, act_id)| (t_id as u8, *act_id))
            })
            .flatten()
    }
    /// Finds transition for fncall
    pub fn get_target_trans_with_for_fncall(
        &self,
        wft: &Template,
        fn_call_ident: FnCallId,
    ) -> Option<(TransitionId, ActivityId)> {
        wft.transitions
            .get(self.current_activity_id as usize)
            .map(|t| {
                t.iter()
                    .enumerate()
                    .find(|(_, act_id)| {
                        if wft.activities[**act_id as usize].as_ref().unwrap().action
                            != ActionType::FnCall
                        {
                            return false;
                        }

                        match wft.activities[**act_id as usize]
                            .as_ref()
                            .unwrap()
                            .action_data
                            .as_ref()
                        {
                            Some(data) => match data {
                                ActionData::FnCall(fncall) => fncall.id == fn_call_ident,
                                ActionData::Event(_) => false,
                            },
                            None => false,
                        }
                    })
                    .map(|(t_id, act_id)| (t_id as u8, *act_id))
            })
            .flatten()
    }

    /// Finds transition for event
    pub fn get_target_trans_with_for_event(
        &self,
        wft: &Template,
        event_code: &EventCode,
    ) -> Option<(TransitionId, ActivityId)> {
        wft.transitions
            .get(self.current_activity_id as usize)
            .map(|t| {
                t.iter()
                    .enumerate()
                    .find(|(_, act_id)| {
                        if wft.activities[**act_id as usize].as_ref().unwrap().action
                            != ActionType::Event
                        {
                            return false;
                        }

                        match wft.activities[**act_id as usize]
                            .as_ref()
                            .unwrap()
                            .action_data
                            .as_ref()
                        {
                            Some(data) => match data {
                                ActionData::FnCall(_) => false,
                                ActionData::Event(e) => e.code == *event_code,
                            },
                            None => false,
                        }
                    })
                    .map(|(t_id, act_id)| (t_id as u8, *act_id))
            })
            .flatten()
    }

    // TODO figure out cond eval and pos_level
    /// Tries to advance to next activity in workflow and updates counter.
    /// Conditions might panics underneath.
    pub fn transition_to_next(
        &mut self,
        activity_id: u8,
        transition_id: u8,
        wft: &Template,
        consts: &Consts,
        wfs: &TemplateSettings,
        settings: &ProposeSettings,
        action_args: &[Vec<DataType>],
        storage_bucket: &StorageBucket,
        pos_level: usize,
    ) -> (ActionResult, Option<Postprocessing>) {
        //TODO switching to finish state
        if self.state == InstanceState::Finished {
            return (ActionResult::Finished, None);
        }

        let transition_settings =
            &wfs.transition_constraints[self.current_activity_id as usize][transition_id as usize];

        // TODO trans and activity cond should be required only validation against storage
        //check transition cond
        let transition_cond_result = match &transition_settings.cond {
            Some(c) => c
                .bind_and_eval(
                    consts,
                    storage_bucket,
                    settings.binds.as_slice(),
                    &action_args[pos_level],
                )
                .try_into_bool()
                .unwrap_or(true),
            None => true,
        };

        if !transition_cond_result {
            return (ActionResult::TransitionCondFailed, None);
        }

        // check transition counter
        if self.transition_counter[self.current_activity_id as usize][transition_id as usize] + 1
            > transition_settings.transition_limit
        {
            return (ActionResult::MaxTransitionLimitReached, None);
        }

        self.transition_counter[self.current_activity_id as usize][transition_id as usize] += 1;
        self.previous_activity_id = self.current_activity_id;
        self.current_activity_id = activity_id;

        // check if we can run this
        let wanted_activity = wft.activities[activity_id as usize].as_ref().unwrap();
        let can_be_exec = match wanted_activity.exec_condition {
            Some(ref e) => e.bind_and_eval(
                consts,
                storage_bucket,
                settings.binds.as_slice(),
                &action_args[pos_level],
            ),
            None => DataType::Bool(true),
        };

        if !can_be_exec.try_into_bool().unwrap() {
            return (ActionResult::ActivityCondFailed, None);
        }

        (ActionResult::Ok, wanted_activity.postprocessing.clone())
    } */
}
