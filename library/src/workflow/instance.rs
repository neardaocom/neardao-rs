use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
};

use super::activity::{Transition, TransitionCounter, TransitionLimit};

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
    pub transition_counters: Vec<Vec<TransitionCounter>>,
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
            transition_counters: Vec::default(),
            template_id,
            awaiting_state: None,
        }
    }

    /// Updates necessary attributes.
    /// Does not check if transition is possible - might panic.
    pub fn transition_next(
        &mut self,
        activity_id: usize,
        activity_actions_count: u8,
        actions_already_done: u8,
    ) {
        let counter_pos = self
            .find_transition_counter_pos(activity_id as u8)
            .expect("fatal - transition not found");
        self.transition_counters[self.current_activity_id as usize]
            .get_mut(counter_pos)
            .unwrap()
            .inc_count();
        self.actions_done_count = actions_already_done;
        self.actions_total = activity_actions_count;
        self.previous_activity_id = self.current_activity_id;
        self.current_activity_id = activity_id as u8;
    }

    pub fn set_current_activity_done(&mut self) {
        self.actions_done_count = self.actions_total;
    }

    /// Inits transition counter.
    /// Requires `settings_transitions` to have same structure as `template_transitions`.
    /// But it should be checked in higher layers.
    pub fn init_transition_counter(
        &mut self,
        template_transitions: &[Vec<Transition>],
        settings_transitions: &[Vec<TransitionLimit>],
    ) {
        self.transition_counters = Vec::with_capacity(template_transitions.len());

        for (i, transition_limit) in template_transitions.iter().enumerate() {
            let mut limits = Vec::with_capacity(template_transitions.len());
            for (j, _) in transition_limit.iter().enumerate() {
                limits.push(TransitionCounter {
                    to: settings_transitions[i][j].to,
                    count: 0,
                    limit: settings_transitions[i][j].limit,
                });
            }
            self.transition_counters.push(limits);
        }
    }

    pub fn check_transition_counter(&self, activity_id: usize) -> bool {
        let counter_pos = self
            .find_transition_counter_pos(activity_id as u8)
            .expect("fatal - transition not found");
        self.transition_counters[self.current_activity_id as usize][counter_pos]
            .is_another_transition_allowed()
    }

    pub fn find_transition<'a>(
        &self,
        transitions: &'a [Vec<Transition>],
        activity_id: usize,
    ) -> Option<&'a Transition> {
        // Current activity is not finished yet.
        if self.actions_done_count != self.actions_total {
            return None;
        }
        transitions
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

    /// Checks if new transition is done to another activity or loop
    pub fn is_new_transition(&self, target_activity_id: usize) -> bool {
        self.current_activity_id as usize == target_activity_id
            && self.actions_done_count == self.actions_total
            || self.current_activity_id as usize != target_activity_id
    }

    /// Finds pos of `TransitionCounter` for target `activity_id`.
    fn find_transition_counter_pos(&self, activity_id: u8) -> Option<usize> {
        self.transition_counters[self.current_activity_id as usize]
            .iter()
            .position(|c| c.to == activity_id)
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
