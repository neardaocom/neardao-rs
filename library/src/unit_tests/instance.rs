use crate::{
    data::workflows::integration::skyward::Skyward1,
    workflow::{
        activity::{Transition, TransitionLimit},
        instance::Instance,
    },
};

const TEMPLATE_ID: u16 = 1;

fn test_data() -> (
    Vec<Vec<Transition>>,
    Vec<Vec<TransitionLimit>>,
    usize,
    Vec<u8>,
) {
    let tpl_data = Skyward1::template(None);
    let tpls_settings = Skyward1::template_settings();
    (
        tpl_data.0.transitions,
        tpls_settings.transition_limits,
        tpl_data.0.activities.len(),
        tpl_data.0.end,
    )
}

#[test]
fn dao_action_new_all() {
    let (tpls_trans, settings_trans, len, end) = test_data();
    let mut instance = Instance::new(TEMPLATE_ID, len, end);
    instance.init_running(tpls_trans.as_slice(), settings_trans.as_slice())
}

#[test]
fn dao_action_new_part() {}

#[test]
fn dao_action_finish_current() {}

#[test]
fn dao_action_rollback_new() {}

#[test]
fn fncall_new_all() {}

#[test]
fn fncall_new_part() {}

#[test]
fn fncall_finish_current() {}

#[test]
fn fncall_rollback_new() {}

#[test]
fn fncall_out_of_order() {}
