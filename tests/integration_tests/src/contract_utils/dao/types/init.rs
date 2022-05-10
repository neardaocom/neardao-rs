use std::collections::HashMap;

use library::workflow::{settings::TemplateSettings, template::Template, types::ObjectMetadata};
use serde::{Deserialize, Serialize};
use workspaces::AccountId;

use crate::utils::{DurationSec, FnCallId, MethodName};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct DaoInit {
    pub token_id: AccountId,
    pub staking_id: AccountId,
    pub total_supply: u32,
    pub decimals: u8,
    pub settings: DaoSettings,
    pub groups: Vec<GroupInput>,
    pub tags: Vec<u16>,
    pub standard_function_calls: Vec<MethodName>,
    pub standard_function_call_metadata: Vec<Vec<ObjectMetadata>>,
    pub function_calls: Vec<FnCallId>,
    pub function_call_metadata: Vec<Vec<ObjectMetadata>>,
    pub workflow_templates: Vec<Template>,
    pub workflow_template_settings: Vec<Vec<TemplateSettings>>,
    pub tick_interval: DurationSec,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct DaoSettings {
    pub name: String,
    pub purpose: String,
    pub tags: Vec<u16>,
    pub dao_admin_account_id: AccountId,
    pub dao_admin_rights: Vec<String>, //TODO should be rights
    pub workflow_provider: AccountId,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct GroupInput {
    pub settings: GroupSettings,
    pub members: Vec<GroupMember>,
    pub member_roles: HashMap<String, Vec<String>>,
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct GroupSettings {
    pub name: String,
    pub leader: Option<AccountId>,
    pub parent_group: u16,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct GroupMember {
    pub account_id: AccountId,
    pub tags: Vec<u16>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct GroupTokenLockInput {
    pub amount: u32,
    pub start_from: u64,
    pub duration: u64,
    pub init_distribution: u32,
    pub unlock_interval: u32,
    pub periods: Vec<UnlockPeriodInput>,
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct GroupOutput {
    pub id: u16,
    pub settings: GroupSettings,
    pub members: Vec<GroupMember>,
}

impl GroupOutput {
    pub fn from_expected(
        id: u16,
        name: String,
        leader: Option<AccountId>,
        parent_group: u16,
        members: Vec<(&AccountId, Vec<u16>)>,
    ) -> Self {
        GroupOutput {
            id,
            settings: GroupSettings {
                name,
                leader,
                parent_group,
            },
            members: members
                .into_iter()
                .map(|m| GroupMember {
                    account_id: m.0.to_owned(),
                    tags: m.1,
                })
                .collect(),
        }
    }
}
