use library::workflow::{settings::TemplateSettings, template::Template, types::ObjectMetadata};
use serde::{Deserialize, Serialize};
use workspaces::AccountId;

use crate::utils::{FnCallId, MethodName};

use super::group::GroupInput;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct DaoInit {
    pub token_id: AccountId,
    pub staking_id: AccountId,
    pub total_supply: u32,
    pub decimals: u8,
    pub settings: Settings,
    pub groups: Vec<GroupInput>,
    pub tags: Vec<u16>,
    pub standard_function_calls: Vec<MethodName>,
    pub standard_function_call_metadata: Vec<Vec<ObjectMetadata>>,
    pub function_calls: Vec<FnCallId>,
    pub function_call_metadata: Vec<Vec<ObjectMetadata>>,
    pub workflow_templates: Vec<Template>,
    pub workflow_template_settings: Vec<Vec<TemplateSettings>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Settings {
    pub name: String,
    pub purpose: String,
    pub tags: Vec<u16>,
    pub dao_admin_account_id: AccountId,
    pub dao_admin_rights: Vec<String>, //TODO should be rights
    pub workflow_provider: AccountId,
    pub resource_provider: AccountId,
    pub scheduler: AccountId,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum UnlockMethod {
    /// All FT immediately unlocked.
    None = 0,
    /// Linear unlocker over specified time period.
    Linear,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct UnlockPeriodInput {
    pub kind: UnlockMethod,
    pub duration: u64,
    pub amount: u32,
}
