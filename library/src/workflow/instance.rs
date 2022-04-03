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
    /// Temporary state during execution when 1..N promises were dispached.
    AwaitingPromise,
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
    pub awaiting_state: Option<AwaitingState>,
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
            awaiting_state: None,
        }
    }

    /// Updates necessary attributes.
    /// Does not check if transition is possible - might panic.
    pub fn transition_next(
        &mut self,
        activity_id: u8,
        activity_actions_count: u8,
        actions_already_done: u8,
    ) {
        self.transition_counter[self.current_activity_id as usize][activity_id as usize] += 1;
        self.actions_done_count = actions_already_done;
        self.actions_total = activity_actions_count;
        self.previous_activity_id = self.current_activity_id;
        self.current_activity_id = activity_id as u8;
    }

    pub fn set_current_activity_done(&mut self) {
        self.actions_done_count = self.actions_total;
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

    pub fn set_new_awaiting_state(
        &mut self,
        activity_id: u8,
        is_new_transition: bool,
        new_activity_actions_count: u8,
        wf_finish: bool,
    ) {
        self.state = InstanceState::AwaitingPromise;
        self.awaiting_state = Some(AwaitingState::new(
            activity_id,
            is_new_transition,
            new_activity_actions_count,
            wf_finish,
        ));
    }

    pub fn unset_awaiting_state(&mut self, new_state: InstanceState) {
        self.state = new_state;
        self.awaiting_state = None;
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct AwaitingState {
    pub is_new_transition: bool,
    pub activity_id: u8,
    pub new_activity_actions_count: u8,
    pub wf_finish: bool,
}

impl AwaitingState {
    pub fn new(
        activity_id: u8,
        is_new_transition: bool,
        new_activity_actions_count: u8,
        wf_finish: bool,
    ) -> Self {
        AwaitingState {
            is_new_transition,
            activity_id,
            new_activity_actions_count,
            wf_finish,
        }
    }
}
