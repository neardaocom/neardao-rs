use std::collections::HashMap;

use library::{
    types::{
        activity_input::{InputHashMap, UserInput},
        datatype::Value,
    },
    workflow::action::{ActionInput, DaoActionOrFnCall},
};
use near_sdk::AccountId;
use workspaces::AccountId as WorkspaceAccountId;

use crate::contract_utils::dao::types::consts::PROVIDER_VIEW_TEMPLATE;

pub struct ActivityInputWorkflowAdd;

impl ActivityInputWorkflowAdd {
    pub fn activity_1(
        provider_id: &WorkspaceAccountId,
        wf_template_id: u16,
    ) -> Vec<Option<ActionInput>> {
        let mut map = HashMap::new();
        map.insert("id".to_string(), Value::U64(wf_template_id as u64));
        let provider_id = AccountId::new_unchecked(provider_id.to_string());
        vec![Some(ActionInput {
            action: DaoActionOrFnCall::FnCall(provider_id, PROVIDER_VIEW_TEMPLATE.to_string()),
            values: UserInput::Map(InputHashMap(map)),
        })]
    }
}
