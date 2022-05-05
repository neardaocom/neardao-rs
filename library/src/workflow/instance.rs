use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
};

use super::activity::{Transition, TransitionCounter, TransitionLimit};

#[derive(
    BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Debug, Clone, Copy,
)]
#[serde(crate = "near_sdk::serde")]
pub enum InstanceState {
    /// Waiting for proposal to be accepted.
    Waiting,
    /// Workflow is running.
    Running,
    /// Temporary state during execution when 1..N promises were dispached but haven't resolved yet or none of them failed.
    AwaitingPromise,
    /// Unrecoverable error happened. Eg. by executing badly defined workflow.
    FatalError,
    /// Workflow was finished and closed.
    Finished,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Instance {
    state: InstanceState,
    last_transition_done_at: u64,
    current_activity_id: u8,
    previous_activity_id: u8,
    transition_counters: Vec<Vec<TransitionCounter>>,
    /// Last activity's count of done actions. < actions.len() during execution.
    actions_done_count: u8,
    actions_total: u8,
    template_id: u16,
    /// Flag with info if its new activity transition.
    transition_to_new_activity: Option<ActivityInfo>,
    end_activities: Vec<u8>,
    /// Flag if current activity can be autofinished.
    current_activity_autofinish: bool,
    dispatched_promises_count: u8,
}

impl Instance {
    pub fn new(template_id: u16, end_activities: Vec<u8>) -> Self {
        Instance {
            state: InstanceState::Waiting,
            last_transition_done_at: 0,
            current_activity_id: 0,
            previous_activity_id: 0,
            actions_done_count: 0,
            actions_total: 0,
            transition_counters: Vec::default(),
            template_id,
            transition_to_new_activity: None,
            end_activities,
            current_activity_autofinish: false,
            dispatched_promises_count: 0,
        }
    }

    /// Inits instance to running state.
    /// Requires `settings_transitions` to have same structure as `template_transitions`.
    pub fn init_running(
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
        self.state = InstanceState::Running;
    }

    /// Registers new activity.
    /// Checks must be done before.
    /// Must be called always when a new activity is about to be executed.
    pub fn register_new_activity(
        &mut self,
        activity_id: u8,
        new_activity_actions_count: u8,
        autofinish: bool,
    ) {
        self.actions_done_count = 0;
        self.transition_to_new_activity = Some(ActivityInfo::new(
            activity_id,
            new_activity_actions_count,
            autofinish,
        ));
    }

    // Update transition counter if transition is possible.
    pub fn update_transition_counter(&mut self, activity_id: usize) -> bool {
        let counter_pos = self
            .find_transition_counter_pos(activity_id as u8)
            .expect("fatal - transition not found");
        let transition = self.transition_counters[self.current_activity_id as usize]
            .get_mut(counter_pos)
            .unwrap();
        if transition.is_another_transition_allowed() {
            transition.inc_count();
            true
        } else {
            false
        }
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

    /// Checks if provided `target_activity_id` activity means transitioning to new activity.
    pub fn is_new_transition(&self, target_activity_id: usize) -> bool {
        self.current_activity_id as usize == target_activity_id
            && self.actions_done_count == self.actions_total
            || self.current_activity_id as usize != target_activity_id
    }

    /// Find pos of `TransitionCounter` for target `activity_id`.
    fn find_transition_counter_pos(&self, activity_id: u8) -> Option<usize> {
        self.transition_counters[self.current_activity_id as usize]
            .iter()
            .position(|c| c.to == activity_id)
    }

    /// Check if action is invalid as next action.
    /// In case of true switches self back to running state and returns true.
    /// Used to secure function call transaction.
    pub fn check_invalid_action(&mut self, action_id: u8) -> bool {
        if self.actions_done_count < action_id {
            self.transition_to_new_activity = None;
            self.state = InstanceState::Running;
            self.dispatched_promises_count = 0;
            true
        } else {
            false
        }
    }

    /// Try to advance next to activity.
    /// If successful then update internal states and return true.
    /// New actions done and timestamp must be updated `try_to_finish` function after.
    pub fn try_to_advance_activity(&mut self) -> bool {
        if let Some(info) = self.transition_to_new_activity.as_ref().take() {
            self.last_transition_done_at = 0;
            self.actions_done_count = 0;
            self.actions_total = info.new_activity_actions_count;
            self.previous_activity_id = self.current_activity_id;
            self.current_activity_id = info.activity_id as u8;
            self.current_activity_autofinish = info.autofinish;
            true
        } else {
            false
        }
    }

    /// Update actions done count and timestamp and try to finish workflow.
    pub fn try_to_finish(
        &mut self,
        new_actions_done_count: u8,
        current_timestamp_sec: u64,
    ) -> bool {
        self.actions_done_count += new_actions_done_count;
        self.last_transition_done_at = current_timestamp_sec;
        if self.dispatched_promises_count == 0
            && self.current_activity_autofinish
            && self.actions_done_count == self.actions_total
            && self.end_activities.contains(&self.current_activity_id)
        {
            self.state = InstanceState::Finished;
            true
        } else {
            false
        }
    }
    pub fn get_state(&self) -> InstanceState {
        self.state
    }

    pub fn await_promises(&mut self, promise_count: u8) {
        self.state = InstanceState::AwaitingPromise;
        self.dispatched_promises_count = promise_count;
    }

    /// At least one of the promises failed.
    /// Set internal state back to running.
    pub fn promise_failed(&mut self) {
        self.state = InstanceState::Running;
        self.transition_to_new_activity = None;
        self.dispatched_promises_count = 0;
    }

    /// Decreases counter for dispatched promises.
    /// Panics if called more times than count of promises dispatched.
    pub fn promise_success(&mut self) {
        self.dispatched_promises_count = self
            .dispatched_promises_count
            .checked_sub(1)
            .expect("fatal - promise count");
        if self.dispatched_promises_count == 0 {
            self.state = InstanceState::Running;
        }
    }

    pub fn set_fatal_error(&mut self) {
        self.state = InstanceState::FatalError;
    }

    pub fn get_current_activity_id(&self) -> u8 {
        self.current_activity_id
    }
    pub fn actions_done_count(&self) -> u8 {
        self.actions_done_count
    }
    pub fn actions_remaining(&self) -> u8 {
        self.actions_total - self.actions_done_count
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct ActivityInfo {
    //pub is_new_transition: bool,
    pub activity_id: u8,
    pub new_activity_actions_count: u8,
    pub autofinish: bool,
}

impl ActivityInfo {
    pub fn new(
        new_activity_id: u8,
        //is_new_transition: bool,
        new_activity_actions_count: u8,
        autofinish: bool,
    ) -> Self {
        ActivityInfo {
            //is_new_transition,
            activity_id: new_activity_id,
            new_activity_actions_count,
            autofinish,
        }
    }
}
