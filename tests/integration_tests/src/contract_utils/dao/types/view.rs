use std::collections::HashMap;

use library::{
    types::datatype::Value,
    workflow::{instance::Instance, settings::TemplateSettings, template::Template},
};
use near_sdk::json_types::U128;
use serde::{Deserialize, Serialize};

use super::proposal::VersionedProposal;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct StorageBalance {
    total: U128,
    available: U128,
}

impl Default for StorageBalance {
    fn default() -> Self {
        Self {
            total: 0.into(),
            available: 0.into(),
        }
    }
}

pub(crate) type ViewInstance = Option<Instance>;
pub(crate) type ViewTemplates = Vec<(u16, (Template, Vec<TemplateSettings>))>;
pub(crate) type ViewProposal = Option<(VersionedProposal, Option<Vec<TemplateSettings>>)>;
pub(crate) type ViewWorkflowStorage = Option<Vec<(String, Value)>>;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive())]
#[serde(crate = "near_sdk::serde")]
pub struct Statistics {
    pub staking_id: String,
    pub token_id: String,
    pub total_delegation_amount: U128,
    pub total_delegators_count: u32,
    pub ft_total_supply: u32,
    pub decimals: u8,
    pub total_members_count: u32,
    pub total_account_balance: U128,
    pub free_account_balance: U128,
}
