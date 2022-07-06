use std::collections::HashMap;

use library::{
    types::Value,
    workflow::action::{ActionInput, ActionInputType},
    workflow::runtime::activity_input::UserInput,
};
use near_sdk::AccountId;
use workspaces::AccountId as WorkspaceAccountId;

use crate::constants::PROVIDER_VIEW_TEMPLATE;

/// Activity inputs for `WfBasicPkg1`.
pub struct ActivityInputWfBasicPkg1;
impl ActivityInputWfBasicPkg1 {
    pub fn activity_1(
        provider_id: &WorkspaceAccountId,
        wf_template_id: u16,
    ) -> Vec<Option<ActionInput>> {
        let mut map = HashMap::new();
        map.insert("id".to_string(), Value::U64(wf_template_id as u64));
        let provider_id = AccountId::new_unchecked(provider_id.to_string());
        vec![Some(ActionInput {
            action: ActionInputType::FnCall(provider_id, PROVIDER_VIEW_TEMPLATE.to_string()),
            values: UserInput::Map(map),
        })]
    }
}
