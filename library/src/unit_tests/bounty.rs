use near_sdk::test_utils::VMContextBuilder;
use near_sdk::testing_env;
use near_sdk::MockedBlockchain;

use crate::data::bounty::{
    workflow_bounty_template_data_1, workflow_bounty_template_settings_data_1,
};

use crate::types::{ActionData, ActionIdent};
use crate::unit_tests::{get_dao_consts, ONE_NEAR};
use crate::utils::validate_args;
use crate::workflow::ActionResult;
use crate::{
    storage::StorageBucket,
    types::DataType,
    workflow::{Instance, InstanceState},
};

pub const USER_ACC_1: &str = "user1.testnet";
pub const COUNCIL_ACC_1: &str = "c1.testnet";

#[test]
fn workflow_bounty_happy_scenario() {
    let mut builder = VMContextBuilder::new();
    testing_env!(builder.build());

    let (wft, fncalls, fn_metadata) = workflow_bounty_template_data_1();
    let (wfs, settings) = workflow_bounty_template_settings_data_1();

    let mut wfi = Instance::new(1, &wft.transitions);
    let mut bucket = StorageBucket::new(b"wf_bounty".to_vec());
    wfi.state = InstanceState::Running;

    // Execute Workflow

    // 1. CheckIn
    let user_args = vec![vec![DataType::String(USER_ACC_1.into())]];

    let (transition_id, activity_id) = wfi
        .get_target_trans_with_for_event(&wft, &"checkin".into())
        .unwrap();

    assert_eq!(activity_id, 1);
    assert_eq!(transition_id, 0);

    let dao_consts = get_dao_consts();

    let result = wfi.transition_to_next(
        activity_id,
        transition_id,
        &wft,
        &dao_consts,
        &wfs[0],
        &settings,
        &user_args,
        &mut bucket,
        0,
    );

    let actual_event_code = match &*wft.activities[activity_id as usize]
        .as_ref()
        .unwrap()
        .action_data
        .as_ref()
        .unwrap()
    {
        ActionData::Event(data) => data.code.clone(),
        _ => panic!("Invalid Data"),
    };

    // activity result check
    assert_eq!(result.0, ActionResult::Ok);
    assert_eq!(wfi.current_activity_id, 1);
    assert_eq!(actual_event_code, "checkin".to_string());

    assert!(validate_args(
        &dao_consts,
        &settings.binds,
        &wft.obj_validators[activity_id as usize - 1].as_slice(),
        &wft.validator_exprs.as_slice(),
        &bucket,
        user_args.as_slice(),
        &[],
        &[],
    ));

    let pp = wft.activities[activity_id as usize]
        .as_ref()
        .unwrap()
        .postprocessing
        .as_ref()
        .unwrap();

    let inner_value = pp.try_to_get_inner_value(user_args.as_slice(), settings.binds.as_slice());
    assert_eq!(inner_value, Some(DataType::String(USER_ACC_1.into())));

    let pp_value = pp.clone().postprocess(vec![], inner_value, &mut bucket);
    assert_eq!(pp_value, Some(DataType::String(USER_ACC_1.into())));

    bucket.add_data(&"pp_1".into(), &DataType::String(USER_ACC_1.into()));

    // 2. Approved by a council member

    let user_args = vec![vec![
        DataType::String(COUNCIL_ACC_1.into()),
        DataType::Bool(true),
    ]];

    let (transition_id, activity_id) = wfi
        .get_target_trans_with_for_event(&wft, &"approve".into())
        .unwrap();

    assert_eq!(activity_id, 3);
    assert_eq!(transition_id, 1);

    let dao_consts = get_dao_consts();

    let result = wfi.transition_to_next(
        activity_id,
        transition_id,
        &wft,
        &dao_consts,
        &wfs[0],
        &settings,
        &user_args,
        &mut bucket,
        0,
    );

    let actual_event_code = match &*wft.activities[activity_id as usize]
        .as_ref()
        .unwrap()
        .action_data
        .as_ref()
        .unwrap()
    {
        ActionData::Event(data) => data.code.clone(),
        _ => panic!("Invalid Data"),
    };

    // activity result check
    assert_eq!(result.0, ActionResult::Ok);
    assert_eq!(wfi.current_activity_id, 3);
    assert_eq!(actual_event_code, "approve".to_string());

    assert!(validate_args(
        &dao_consts,
        &settings.binds,
        &wft.obj_validators[activity_id as usize - 1].as_slice(),
        &wft.validator_exprs.as_slice(),
        &bucket,
        user_args.as_slice(),
        &[],
        &[],
    ));

    let pp = wft.activities[activity_id as usize]
        .as_ref()
        .unwrap()
        .postprocessing
        .as_ref()
        .unwrap();

    let inner_value = pp.try_to_get_inner_value(user_args.as_slice(), settings.binds.as_slice());
    assert_eq!(inner_value, Some(DataType::Bool(true)));

    let pp_value = pp.clone().postprocess(vec![], inner_value, &mut bucket);
    assert_eq!(pp_value, Some(DataType::Bool(true)));

    bucket.add_data(&"pp_3".into(), &DataType::Bool(true));

    // 3. Task done by the user

    let user_args = vec![vec![
        DataType::String(USER_ACC_1.into()),
        DataType::String("done task link ...".into()),
    ]];

    let (transition_id, activity_id) = wfi
        .get_target_trans_with_for_event(&wft, &"done".into())
        .unwrap();

    assert_eq!(activity_id, 4);
    assert_eq!(transition_id, 2);

    let dao_consts = get_dao_consts();

    let result = wfi.transition_to_next(
        activity_id,
        transition_id,
        &wft,
        &dao_consts,
        &wfs[0],
        &settings,
        &user_args,
        &mut bucket,
        0,
    );

    let actual_event_code = match &*wft.activities[activity_id as usize]
        .as_ref()
        .unwrap()
        .action_data
        .as_ref()
        .unwrap()
    {
        ActionData::Event(data) => data.code.clone(),
        _ => panic!("Invalid Data"),
    };

    // activity result check
    assert_eq!(result.0, ActionResult::Ok);
    assert_eq!(wfi.current_activity_id, 4);
    assert_eq!(actual_event_code, "done".to_string());

    assert!(validate_args(
        &dao_consts,
        &settings.binds,
        &wft.obj_validators[activity_id as usize - 1].as_slice(),
        &wft.validator_exprs.as_slice(),
        &bucket,
        user_args.as_slice(),
        &[],
        &[],
    ));

    let pp = wft.activities[activity_id as usize]
        .as_ref()
        .unwrap()
        .postprocessing
        .as_ref()
        .unwrap();

    let inner_value = pp.try_to_get_inner_value(user_args.as_slice(), settings.binds.as_slice());
    assert_eq!(
        inner_value,
        Some(DataType::String("done task link ...".into()))
    );

    let pp_value = pp.clone().postprocess(vec![], inner_value, &mut bucket);
    assert_eq!(
        pp_value,
        Some(DataType::String("done task link ...".into()))
    );

    // 4. Council confirmed done the task with some note

    let user_args = vec![vec![
        DataType::String(COUNCIL_ACC_1.into()),
        DataType::String("great work, 5/5".into()),
    ]];

    let (transition_id, activity_id) = wfi
        .get_target_trans_with_for_event(&wft, &"done_approve".into())
        .unwrap();

    assert_eq!(activity_id, 5);
    assert_eq!(transition_id, 0);

    let dao_consts = get_dao_consts();

    let result = wfi.transition_to_next(
        activity_id,
        transition_id,
        &wft,
        &dao_consts,
        &wfs[0],
        &settings,
        &user_args,
        &mut bucket,
        0,
    );

    let actual_event_code = match &*wft.activities[activity_id as usize]
        .as_ref()
        .unwrap()
        .action_data
        .as_ref()
        .unwrap()
    {
        ActionData::Event(data) => data.code.clone(),
        _ => panic!("Invalid Data"),
    };

    // activity result check
    assert_eq!(result.0, ActionResult::Ok);
    assert_eq!(wfi.current_activity_id, 5);
    assert_eq!(actual_event_code, "done_approve".to_string());

    assert!(validate_args(
        &dao_consts,
        &settings.binds,
        &wft.obj_validators[activity_id as usize - 1].as_slice(),
        &wft.validator_exprs.as_slice(),
        &bucket,
        user_args.as_slice(),
        &[],
        &[],
    ));

    let pp = wft.activities[activity_id as usize]
        .as_ref()
        .unwrap()
        .postprocessing
        .as_ref()
        .unwrap();

    let inner_value = pp.try_to_get_inner_value(user_args.as_slice(), settings.binds.as_slice());
    assert_eq!(
        inner_value,
        Some(DataType::String("great work, 5/5".into()))
    );

    let pp_value = pp.clone().postprocess(vec![], inner_value, &mut bucket);
    assert_eq!(pp_value, Some(DataType::String("great work, 5/5".into())));

    // 5. Payout from council

    let user_args = vec![vec![
        DataType::String(USER_ACC_1.into()),
        DataType::U128((5 * ONE_NEAR).into()),
    ]];

    let (transition_id, activity_id) = wfi
        .get_target_trans_with_for_dao_action(&wft, ActionIdent::TreasurySendNear)
        .unwrap();

    assert_eq!(activity_id, 6);
    assert_eq!(transition_id, 0);

    let dao_consts = get_dao_consts();

    let result = wfi.transition_to_next(
        activity_id,
        transition_id,
        &wft,
        &dao_consts,
        &wfs[0],
        &settings,
        &user_args,
        &mut bucket,
        0,
    );

    let actual_action_ident = &wft.activities[activity_id as usize]
        .as_ref()
        .unwrap()
        .action; // activity result check

    assert_eq!(result.0, ActionResult::Ok);
    assert_eq!(wfi.current_activity_id, 6);
    assert_eq!(*actual_action_ident, ActionIdent::TreasurySendNear);

    assert!(validate_args(
        &dao_consts,
        &settings.binds,
        &wft.obj_validators[activity_id as usize - 1].as_slice(),
        &wft.validator_exprs.as_slice(),
        &bucket,
        user_args.as_slice(),
        &[],
        &[],
    ));

    let pp = wft.activities[activity_id as usize]
        .as_ref()
        .unwrap()
        .postprocessing
        .as_ref();

    assert_eq!(pp, None);
}
