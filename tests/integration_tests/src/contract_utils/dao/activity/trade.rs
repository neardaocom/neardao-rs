use std::collections::HashMap;

use library::{
    types::activity_input::UserInput,
    workflow::action::{ActionInput, DaoActionOrFnCall},
};

/// Activity inputs for `Trade1`.
pub struct ActivityInputTrade1;
impl ActivityInputTrade1 {
    pub fn activity_1() -> Vec<Option<ActionInput>> {
        vec![Some(ActionInput {
            action: DaoActionOrFnCall::SendNear,
            values: UserInput::Map(HashMap::new()),
        })]
    }
}
