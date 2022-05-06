//! Workflow and its data for interaction with Skyward finance (https://app.skyward.finance).
//! Reference: https://github.com/skyward-finance/contracts/tree/master/skyward

use std::collections::HashMap;

use library::{
    data::workflows::integration::skyward::{
        SKYWARD_FNCALL1_NAME, SKYWARD_FNCALL2_NAME, SKYWARD_FNCALL3_NAME, SKYWARD_FNCALL4_NAME,
        SKYWARD_FNCALL5_NAME,
    },
    types::{
        activity_input::{ActivityInput, UserInput},
        datatype::Value,
    },
    workflow::action::{ActionInput, DaoActionOrFnCall},
};
use near_sdk::AccountId;
use workspaces::AccountId as WorkspaceAccountId;

/// Activity inputs for `Skyward1` workflow.
pub struct ActivityInputSkyward1;
impl ActivityInputSkyward1 {
    pub fn activity_1(skyward_id: &WorkspaceAccountId) -> Vec<Option<ActionInput>> {
        let skyward_id: AccountId = AccountId::new_unchecked(skyward_id.to_string());
        vec![Some(ActionInput {
            action: DaoActionOrFnCall::FnCall(skyward_id, SKYWARD_FNCALL1_NAME.to_string()),
            values: UserInput::Map(HashMap::default()),
        })]
    }
    pub fn activity_2(
        wnear_id: &WorkspaceAccountId,
        token_id: &WorkspaceAccountId,
    ) -> Vec<Option<ActionInput>> {
        let wnear_id: AccountId = AccountId::new_unchecked(wnear_id.to_string());
        let token_id: AccountId = AccountId::new_unchecked(token_id.to_string());
        vec![
            Some(ActionInput {
                action: DaoActionOrFnCall::FnCall(
                    wnear_id.clone(),
                    SKYWARD_FNCALL2_NAME.to_string(),
                ),
                values: UserInput::Map(HashMap::default()),
            }),
            Some(ActionInput {
                action: DaoActionOrFnCall::FnCall(
                    token_id.clone(),
                    SKYWARD_FNCALL3_NAME.to_string(),
                ),
                values: UserInput::Map(HashMap::default()),
            }),
        ]
    }

    pub fn activity_3(token_id: &WorkspaceAccountId) -> Vec<Option<ActionInput>> {
        let token_id: AccountId = AccountId::new_unchecked(token_id.to_string());
        vec![Some(ActionInput {
            action: DaoActionOrFnCall::FnCall(token_id.clone(), SKYWARD_FNCALL4_NAME.to_string()),
            values: UserInput::Map(HashMap::default()),
        })]
    }
    pub fn activity_4(
        skyward_id: &WorkspaceAccountId,
        sale_title: String,
        sale_url: String,
    ) -> Vec<Option<ActionInput>> {
        let skyward_id: AccountId = AccountId::new_unchecked(skyward_id.to_string());
        let mut map = HashMap::new();
        map.set("sale.title".into(), Value::String(sale_title));
        map.set("sale.url".into(), Value::String(sale_url));
        vec![Some(ActionInput {
            action: DaoActionOrFnCall::FnCall(skyward_id, SKYWARD_FNCALL5_NAME.to_string()),
            values: UserInput::Map(map),
        })]
    }
}
