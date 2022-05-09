use data::workflow::integration::skyward::Skyward1;
use library::workflow::{
    activity::{Activity, Transition, TransitionLimit},
    instance::{Instance, InstanceState},
};

const TEMPLATE_ID: u16 = 1;

// TODO: Get better test data.
fn test_data() -> (
    Vec<Vec<Transition>>,
    Vec<Vec<TransitionLimit>>,
    usize,
    Vec<u8>,
    Vec<Activity>,
) {
    let tpl_data = Skyward1::template(None);
    let tpls_settings = Skyward1::template_settings();
    (
        tpl_data.0.transitions,
        tpls_settings.transition_limits,
        tpl_data.0.activities.len(),
        tpl_data.0.end,
        tpl_data.0.activities,
    )
}

#[test]
fn sync_scenario() {
    let (tpls_trans, settings_trans, len, end, activities) = test_data();
    let mut instance = Instance::new(TEMPLATE_ID, len, end);
    assert_eq!(instance.get_state(), InstanceState::Waiting);
    assert_eq!(instance.get_current_activity_id(), 0);
    assert_eq!(instance.actions_done_count(), 0);
    instance.init_running(tpls_trans.as_slice(), settings_trans.as_slice());
    assert_eq!(instance.get_state(), InstanceState::Running);
    assert_eq!(instance.get_current_activity_id(), 0);
    assert_eq!(instance.actions_done_count(), 0);

    // First activity.
    let activity_id = 1;
    let activity_actions_len = activities[activity_id]
        .activity_as_ref()
        .unwrap()
        .actions
        .len() as u8;
    assert!(instance.is_new_transition(activity_id));
    assert!(instance
        .find_transition(tpls_trans.as_slice(), activity_id)
        .is_some());
    assert!(instance.update_transition_counter(activity_id));
    instance.register_new_activity(activity_id as u8, activity_actions_len, false);
    assert_eq!(instance.get_state(), InstanceState::Running);
    assert_eq!(instance.get_current_activity_id(), activity_id as u8);
    assert_eq!(instance.actions_done_count(), 0);
    assert!(!instance.new_actions_done(1, 0));
    assert_eq!(instance.get_state(), InstanceState::Running);
    assert_eq!(instance.get_current_activity_id(), activity_id as u8);
    assert_eq!(instance.actions_done_count(), 1);
    assert_eq!(instance.actions_remaining(), 0);

    // Second activity - only one action.
    let activity_id = 2;
    let activity_actions_len = activities[activity_id]
        .activity_as_ref()
        .unwrap()
        .actions
        .len() as u8;
    assert!(instance.is_new_transition(activity_id));
    assert!(instance
        .find_transition(tpls_trans.as_slice(), activity_id)
        .is_some());
    assert!(instance.update_transition_counter(activity_id));
    instance.register_new_activity(activity_id as u8, activity_actions_len, false);
    assert_eq!(instance.get_state(), InstanceState::Running);
    assert_eq!(instance.get_current_activity_id(), activity_id as u8);
    assert_eq!(instance.actions_done_count(), 0);
    assert!(!instance.new_actions_done(1, 0));
    assert_eq!(instance.get_state(), InstanceState::Running);
    assert_eq!(instance.get_current_activity_id(), activity_id as u8);
    assert_eq!(instance.actions_done_count(), 1);
    assert_eq!(instance.actions_remaining(), 1);

    // Second activity - second action.
    assert!(!instance.is_new_transition(activity_id));
    assert_eq!(instance.get_state(), InstanceState::Running);
    assert_eq!(instance.get_current_activity_id(), activity_id as u8);
    assert_eq!(instance.actions_done_count(), 1);
    assert_eq!(instance.actions_remaining(), 1);
    assert!(!instance.new_actions_done(1, 0));
    assert_eq!(instance.get_state(), InstanceState::Running);
    assert_eq!(instance.get_current_activity_id(), activity_id as u8);
    assert_eq!(instance.actions_done_count(), 2);
    assert_eq!(instance.actions_remaining(), 0);
}

// Modified sync scenario.
#[test]
#[should_panic = "fatal - invalid use"]
fn sync_rollback_new() {
    let (tpls_trans, settings_trans, len, end, activities) = test_data();
    let mut instance = Instance::new(TEMPLATE_ID, len, end);
    instance.init_running(tpls_trans.as_slice(), settings_trans.as_slice());

    // First activity.
    let activity_id = 1;
    let activity_actions_len = activities[activity_id]
        .activity_as_ref()
        .unwrap()
        .actions
        .len() as u8;
    assert!(instance.is_new_transition(activity_id));
    assert!(instance
        .find_transition(tpls_trans.as_slice(), activity_id)
        .is_some());
    assert!(instance.update_transition_counter(activity_id));
    instance.register_new_activity(activity_id as u8, activity_actions_len, false);
    assert_eq!(instance.get_state(), InstanceState::Running);
    assert_eq!(instance.get_current_activity_id(), activity_id as u8);
    assert_eq!(instance.actions_done_count(), 0);
    assert!(!instance.new_actions_done(0, 0));
}

#[test]
fn async_scenario() {
    let (tpls_trans, settings_trans, len, end, activities) = test_data();
    let mut instance = Instance::new(TEMPLATE_ID, len, end);
    assert_eq!(instance.get_state(), InstanceState::Waiting);
    assert_eq!(instance.get_current_activity_id(), 0);
    assert_eq!(instance.actions_done_count(), 0);
    instance.init_running(tpls_trans.as_slice(), settings_trans.as_slice());
    assert_eq!(instance.get_state(), InstanceState::Running);
    assert_eq!(instance.get_current_activity_id(), 0);
    assert_eq!(instance.actions_done_count(), 0);

    // First activity.
    let activity_id = 1;
    let activity_actions_len = activities[activity_id]
        .activity_as_ref()
        .unwrap()
        .actions
        .len() as u8;
    assert!(instance.is_new_transition(activity_id));
    assert!(instance
        .find_transition(tpls_trans.as_slice(), activity_id)
        .is_some());
    assert!(instance.update_transition_counter(activity_id));
    instance.register_new_activity(activity_id as u8, activity_actions_len, false);
    instance.await_promises(1);
    assert_eq!(instance.get_state(), InstanceState::Awaiting);
    assert_eq!(instance.get_current_activity_id(), activity_id as u8);
    assert_eq!(instance.actions_done_count(), 0);
    instance.promise_success();
    assert!(!instance.new_actions_done(1, 0));
    assert_eq!(instance.get_state(), InstanceState::Running);
    assert_eq!(instance.get_current_activity_id(), activity_id as u8);
    assert_eq!(instance.actions_done_count(), 1);
    assert_eq!(instance.actions_remaining(), 0);

    // Second activity - only one action.
    let activity_id = 2;
    let activity_actions_len = activities[activity_id]
        .activity_as_ref()
        .unwrap()
        .actions
        .len() as u8;
    assert!(instance.is_new_transition(activity_id));
    assert!(instance
        .find_transition(tpls_trans.as_slice(), activity_id)
        .is_some());
    assert!(instance.update_transition_counter(activity_id));
    instance.register_new_activity(activity_id as u8, activity_actions_len, false);
    instance.await_promises(1);
    assert_eq!(instance.get_state(), InstanceState::Awaiting);
    assert_eq!(instance.get_current_activity_id(), activity_id as u8);
    assert_eq!(instance.actions_done_count(), 0);
    instance.promise_success();
    assert!(!instance.new_actions_done(1, 0));
    assert_eq!(instance.get_state(), InstanceState::Running);
    assert_eq!(instance.get_current_activity_id(), activity_id as u8);
    assert_eq!(instance.actions_done_count(), 1);
    assert_eq!(instance.actions_remaining(), 1);

    // Second activity - second action.
    assert!(!instance.is_new_transition(activity_id));
    assert_eq!(instance.get_state(), InstanceState::Running);
    assert_eq!(instance.get_current_activity_id(), activity_id as u8);
    assert_eq!(instance.actions_done_count(), 1);
    assert_eq!(instance.actions_remaining(), 1);
    instance.await_promises(1);
    assert_eq!(instance.get_state(), InstanceState::Awaiting);
    assert_eq!(instance.actions_done_count(), 1);
    assert_eq!(instance.actions_remaining(), 1);
    instance.promise_success();
    assert!(!instance.new_actions_done(1, 0));
    assert_eq!(instance.get_state(), InstanceState::Running);
    assert_eq!(instance.get_current_activity_id(), activity_id as u8);
    assert_eq!(instance.get_current_activity_id(), activity_id as u8);
    assert_eq!(instance.actions_done_count(), 2);
    assert_eq!(instance.actions_remaining(), 0);
}

// Modified async scenario.
#[test]
fn async_rollback_new() {
    let (tpls_trans, settings_trans, len, end, activities) = test_data();
    let mut instance = Instance::new(TEMPLATE_ID, len, end);
    instance.init_running(tpls_trans.as_slice(), settings_trans.as_slice());

    // First activity.
    let activity_id = 1;
    let activity_actions_len = activities[activity_id]
        .activity_as_ref()
        .unwrap()
        .actions
        .len() as u8;
    assert!(instance.is_new_transition(activity_id));
    assert!(instance
        .find_transition(tpls_trans.as_slice(), activity_id)
        .is_some());
    assert!(instance.update_transition_counter(activity_id));
    instance.register_new_activity(activity_id as u8, activity_actions_len, false);
    instance.await_promises(2);
    assert_eq!(instance.get_state(), InstanceState::Awaiting);
    assert_eq!(instance.get_current_activity_id(), activity_id as u8);
    assert_eq!(instance.actions_done_count(), 0);
    instance.promise_failed();
    assert_eq!(instance.get_state(), InstanceState::Running);
    assert_eq!(instance.get_current_activity_id(), 0);
    assert_eq!(instance.actions_done_count(), 0);
    assert_eq!(instance.actions_remaining(), 0);
}

// Modified async scenario.
#[test]
fn async_promises_out_of_order() {
    let (tpls_trans, settings_trans, len, end, activities) = test_data();
    let mut instance = Instance::new(TEMPLATE_ID, len, end);
    instance.init_running(tpls_trans.as_slice(), settings_trans.as_slice());

    // First activity.
    let activity_id = 1;
    let activity_actions_len = activities[activity_id]
        .activity_as_ref()
        .unwrap()
        .actions
        .len() as u8;
    assert!(instance.is_new_transition(activity_id));
    assert!(instance
        .find_transition(tpls_trans.as_slice(), activity_id)
        .is_some());
    assert!(instance.update_transition_counter(activity_id));
    instance.register_new_activity(activity_id as u8, activity_actions_len, false);
    instance.await_promises(2);
    assert_eq!(instance.get_state(), InstanceState::Awaiting);
    assert_eq!(instance.get_current_activity_id(), activity_id as u8);
    assert_eq!(instance.actions_done_count(), 0);
    assert!(instance.check_invalid_action(1));
    assert_eq!(instance.get_state(), InstanceState::Running);
    assert_eq!(instance.get_current_activity_id(), 0);
    assert_eq!(instance.actions_done_count(), 0);
    assert_eq!(instance.actions_remaining(), 0);
}
