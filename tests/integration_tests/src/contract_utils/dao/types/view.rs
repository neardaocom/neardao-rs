use library::{
    types::datatype::Value,
    workflow::{
        instance::Instance,
        settings::{ProposeSettings, TemplateSettings},
        template::Template,
    },
};
use near_sdk::json_types::U128;
use serde::{Deserialize, Serialize};

use super::proposal::VProposal;

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
pub(crate) type ViewProposal = Option<(VProposal, Option<Vec<TemplateSettings>>)>;
pub(crate) type ViewWorkflowStorage = Option<Vec<(String, Value)>>;
