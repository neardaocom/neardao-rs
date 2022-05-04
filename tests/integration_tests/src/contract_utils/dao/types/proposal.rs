use std::collections::HashMap;

use library::workflow::{
    instance::Instance,
    settings::{ProposeSettings, TemplateSettings},
};
use serde::{Deserialize, Serialize};
use workspaces::AccountId;

use crate::utils::TimestampSec;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ProposalType {
    WorkflowAdd(WorkflowAddOptions),
    Skyward(SkywardOptions),
}

pub(crate) type ViewProposal = Option<(VProposal, Option<Vec<TemplateSettings>>)>;
pub(crate) type Votes = HashMap<AccountId, u8>;
pub(crate) type ViewInstance = Option<(Instance, ProposeSettings)>;

/// Proposal settings for WorkflowAdd.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct WorkflowAddOptions {
    pub id: u16,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SkywardOptions {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ProposalCreateInput {
    desc: ResourceType,
    template_id: u16,
    template_settings_id: u8,
    propose_settings: ProposeSettings,
    template_settings: Option<Vec<TemplateSettings>>,
}

impl ProposalCreateInput {
    pub fn new(
        desc: Option<ResourceType>,
        template_id: u16,
        template_settings_id: u8,
        propose_settings: ProposeSettings,
        template_settings: Option<Vec<TemplateSettings>>,
    ) -> Self {
        ProposalCreateInput {
            desc: desc.unwrap_or_default(),
            template_id,
            template_settings_id,
            propose_settings,
            template_settings,
        }
    }

    pub fn default(
        template_id: u16,
        propose_settings: ProposeSettings,
        template_settings: Option<TemplateSettings>,
    ) -> Self {
        ProposalCreateInput {
            desc: ResourceType::default(),
            template_id,
            template_settings_id: 0,
            propose_settings,
            template_settings: template_settings.map(|s| Some(vec![s])).flatten(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum ResourceType {
    Text(String),
}

impl Default for ResourceType {
    fn default() -> Self {
        Self::Text("default".into())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct Proposal {
    pub desc: u32,
    pub created: TimestampSec,
    pub created_by: AccountId,
    pub votes: HashMap<AccountId, u8>,
    pub state: ProposalState,
    pub workflow_id: u16,
    pub workflow_settings_id: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum ProposalState {
    InProgress,
    Invalid,
    Spam,
    Rejected,
    Accepted,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum VProposal {
    Curr(Proposal),
}

impl From<VProposal> for Proposal {
    fn from(v: VProposal) -> Self {
        match v {
            VProposal::Curr(v) => v,
        }
    }
}
