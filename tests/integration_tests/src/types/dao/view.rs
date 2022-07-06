use library::{
    types::Value,
    workflow::{instance::Instance, settings::TemplateSettings, template::Template},
};
use near_sdk::json_types::U128;
use serde::{Deserialize, Serialize};

use super::proposal::VersionedProposal;

pub type ViewInstance = Option<Instance>;
pub type ViewTemplates = Vec<(u16, (Template, Vec<TemplateSettings>))>;
pub type ViewProposal = Option<(VersionedProposal, Option<Vec<TemplateSettings>>)>;
pub type ViewWorkflowStorage = Option<Vec<(String, Value)>>;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive())]
#[serde(crate = "near_sdk::serde")]
pub struct Statistics {
    pub version: u8,
    pub total_delegation_amount: U128,
    pub total_delegators_count: u32,
    pub ft_total_supply: u32,
    pub decimals: u8,
    pub total_members_count: u32,
    pub total_account_balance: U128,
    pub free_account_balance: U128,
}
