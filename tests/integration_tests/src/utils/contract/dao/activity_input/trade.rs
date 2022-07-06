use std::collections::HashMap;

use library::workflow::{
    action::{ActionInput, ActionInputType},
    runtime::activity_input::UserInput,
};

/// Activity inputs for `Trade1`.
pub struct ActivityInputTrade1;
impl ActivityInputTrade1 {
    pub fn activity_1() -> Vec<Option<ActionInput>> {
        vec![Some(ActionInput {
            action: ActionInputType::SendNear,
            values: UserInput::Map(HashMap::new()),
        })]
    }
}
