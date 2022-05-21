use library::{
    types::activity_input::UserInput,
    workflow::action::{ActionInput, ActionInputType},
};
use std::collections::HashMap;

pub struct ActivityInputTestOptionalActions;
impl ActivityInputTestOptionalActions {
    pub fn activity_1_complete() -> Vec<Option<ActionInput>> {
        vec![
            Some(ActionInput {
                action: ActionInputType::Event("event_1".into()),
                values: UserInput::Map(HashMap::default()),
            }),
            Some(ActionInput {
                action: ActionInputType::Event("event_2".into()),
                values: UserInput::Map(HashMap::default()),
            }),
            Some(ActionInput {
                action: ActionInputType::Event("event_3".into()),
                values: UserInput::Map(HashMap::default()),
            }),
            Some(ActionInput {
                action: ActionInputType::Event("event_4".into()),
                values: UserInput::Map(HashMap::default()),
            }),
        ]
    }
    pub fn activity_1_action_0() -> Vec<Option<ActionInput>> {
        vec![Some(ActionInput {
            action: ActionInputType::Event("event_1".into()),
            values: UserInput::Map(HashMap::default()),
        })]
    }
    pub fn activity_1_action_1() -> Vec<Option<ActionInput>> {
        vec![Some(ActionInput {
            action: ActionInputType::Event("event_2".into()),
            values: UserInput::Map(HashMap::default()),
        })]
    }
    pub fn activity_1_action_2() -> Vec<Option<ActionInput>> {
        vec![Some(ActionInput {
            action: ActionInputType::Event("event_3".into()),
            values: UserInput::Map(HashMap::default()),
        })]
    }
    pub fn activity_1_action_3() -> Vec<Option<ActionInput>> {
        vec![
            None,
            None,
            None,
            Some(ActionInput {
                action: ActionInputType::Event("event_4".into()),
                values: UserInput::Map(HashMap::default()),
            }),
        ]
    }
    pub fn activity_2_complete_optional_missing() -> Vec<Option<ActionInput>> {
        vec![
            Some(ActionInput {
                action: ActionInputType::SendNear,
                values: UserInput::Map(HashMap::default()),
            }),
            None,
            None,
            Some(ActionInput {
                action: ActionInputType::SendNear,
                values: UserInput::Map(HashMap::default()),
            }),
        ]
    }
    pub fn activity_2_complete_rest() -> Vec<Option<ActionInput>> {
        vec![
            None,
            None,
            None,
            Some(ActionInput {
                action: ActionInputType::SendNear,
                values: UserInput::Map(HashMap::default()),
            }),
        ]
    }
    pub fn activity_2_complete() -> Vec<Option<ActionInput>> {
        vec![
            Some(ActionInput {
                action: ActionInputType::SendNear,
                values: UserInput::Map(HashMap::default()),
            }),
            Some(ActionInput {
                action: ActionInputType::SendNear,
                values: UserInput::Map(HashMap::default()),
            }),
            Some(ActionInput {
                action: ActionInputType::SendNear,
                values: UserInput::Map(HashMap::default()),
            }),
            Some(ActionInput {
                action: ActionInputType::SendNear,
                values: UserInput::Map(HashMap::default()),
            }),
        ]
    }
    pub fn activity_2_action_0() -> Vec<Option<ActionInput>> {
        vec![Some(ActionInput {
            action: ActionInputType::SendNear,
            values: UserInput::Map(HashMap::default()),
        })]
    }
    pub fn activity_2_action_1() -> Vec<Option<ActionInput>> {
        vec![Some(ActionInput {
            action: ActionInputType::SendNear,
            values: UserInput::Map(HashMap::default()),
        })]
    }
    pub fn activity_2_action_2() -> Vec<Option<ActionInput>> {
        vec![Some(ActionInput {
            action: ActionInputType::SendNear,
            values: UserInput::Map(HashMap::default()),
        })]
    }
    pub fn activity_2_action_3() -> Vec<Option<ActionInput>> {
        vec![Some(ActionInput {
            action: ActionInputType::SendNear,
            values: UserInput::Map(HashMap::default()),
        })]
    }
}
