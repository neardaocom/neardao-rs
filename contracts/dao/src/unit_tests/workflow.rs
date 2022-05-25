use std::collections::HashMap;

use library::{
    types::activity_input::UserInput,
    workflow::{
        action::{ActionData, ActionInput, ActionInputType, DaoActionData, TemplateAction},
        postprocessing::Postprocessing,
        types::DaoActionIdent,
    },
};

use near_sdk::testing_env;

use crate::unit_tests::{get_context_builder, get_default_contract};

fn test_event_action(optional: bool) -> TemplateAction {
    TemplateAction {
        exec_condition: None,
        validators: vec![],
        action_data: ActionData::Action(DaoActionData {
            name: DaoActionIdent::Event,
            code: Some("event".into()),
            expected_input: None,
            required_deposit: None,
            binds: vec![],
        }),
        postprocessing: None,
        optional,
    }
}

fn action_input() -> Option<ActionInput> {
    Some(ActionInput {
        action: ActionInputType::Event("event".into()),
        values: UserInput::Map(HashMap::default()),
    })
}

#[test]
fn activity_check_activity_input() {
    let mut ctx = get_context_builder();
    testing_env!(ctx.build());
    let mut contract = get_default_contract();

    let actions = vec![test_event_action(false)];
    let inputs = vec![action_input()];
    assert!(contract.check_activity_input(&actions, &inputs, 0));

    let actions = vec![test_event_action(true)];
    let inputs = vec![None];
    assert!(contract.check_activity_input(&actions, &inputs, 0));

    let actions = vec![
        test_event_action(true),
        test_event_action(true),
        test_event_action(true),
    ];
    let inputs = vec![None];
    assert!(contract.check_activity_input(&actions, &inputs, 2));

    let actions = vec![
        test_event_action(true),
        test_event_action(true),
        test_event_action(false),
    ];
    let inputs = vec![action_input()];
    assert!(contract.check_activity_input(&actions, &inputs, 2));

    let actions = vec![
        test_event_action(true),
        test_event_action(true),
        test_event_action(true),
    ];
    let inputs = vec![None, None, None];
    assert!(contract.check_activity_input(&actions, &inputs, 0));

    let actions = vec![
        test_event_action(false),
        test_event_action(true),
        test_event_action(true),
        test_event_action(false),
    ];
    let inputs = vec![action_input(), None, None, action_input()];
    assert!(contract.check_activity_input(&actions, &inputs, 0));
}

#[test]
#[should_panic]
fn activity_check_activity_input_missing_action_input_1() {
    let mut ctx = get_context_builder();
    testing_env!(ctx.build());
    let mut contract = get_default_contract();

    let actions = vec![
        test_event_action(true),
        test_event_action(true),
        test_event_action(true),
    ];
    let inputs = vec![None, None];
    assert!(!contract.check_activity_input(&actions, &inputs, 0));
}

#[test]
#[should_panic]
fn activity_check_activity_input_missing_action_input_2() {
    let mut ctx = get_context_builder();
    testing_env!(ctx.build());
    let mut contract = get_default_contract();

    let actions = vec![
        test_event_action(true),
        test_event_action(true),
        test_event_action(false),
    ];
    let inputs = vec![None, None];
    assert!(!contract.check_activity_input(&actions, &inputs, 0));
}

#[test]
#[should_panic]
fn activity_check_activity_input_missing_action_input_3() {
    let mut ctx = get_context_builder();
    testing_env!(ctx.build());
    let mut contract = get_default_contract();

    let actions = vec![
        test_event_action(true),
        test_event_action(true),
        test_event_action(true),
    ];
    let inputs = vec![None];
    assert!(!contract.check_activity_input(&actions, &inputs, 1));
}

#[test]
fn activity_check_activity_input_invalid() {
    let mut ctx = get_context_builder();
    testing_env!(ctx.build());
    let mut contract = get_default_contract();

    let actions = vec![test_event_action(false)];
    let inputs = vec![None];
    assert!(!contract.check_activity_input(&actions, &inputs, 0));

    let actions = vec![
        test_event_action(true),
        test_event_action(true),
        test_event_action(false),
    ];
    let inputs = vec![None];
    assert!(!contract.check_activity_input(&actions, &inputs, 2));

    let actions = vec![
        test_event_action(false),
        test_event_action(true),
        test_event_action(true),
        test_event_action(false),
    ];
    let inputs = vec![action_input(), None, None, None];
    assert!(!contract.check_activity_input(&actions, &inputs, 0));
}
