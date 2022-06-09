use std::collections::HashMap;

use library::workflow::settings::{ProposeSettings, TemplateSettings};
use near_sdk::json_types::U128;
use serde::{Deserialize, Serialize};
use workspaces::AccountId;

use crate::utils::TimestampSec;

use super::Media;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ProposalType {
    WorkflowAdd(WorkflowAddOptions),
    Skyward(SkywardOptions),
}

pub type Votes = HashMap<AccountId, u8>;

/// Proposal settings for WorkflowAdd.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct WorkflowAddOptions {
    pub id: u16,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SkywardOptions {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ProposalCreateInput {
    description: Option<Media>,
    template_id: u16,
    template_settings_id: u8,
    propose_settings: ProposeSettings,
    template_settings: Option<Vec<TemplateSettings>>,
    scheduler_msg: Option<String>,
}

impl ProposalCreateInput {
    pub fn new(
        description: Option<Media>,
        template_id: u16,
        template_settings_id: u8,
        propose_settings: ProposeSettings,
        template_settings: Option<Vec<TemplateSettings>>,
    ) -> Self {
        ProposalCreateInput {
            description,
            template_id,
            template_settings_id,
            propose_settings,
            template_settings,
            scheduler_msg: None,
        }
    }

    pub fn default(
        template_id: u16,
        propose_settings: ProposeSettings,
        template_settings: Option<Vec<TemplateSettings>>,
    ) -> Self {
        ProposalCreateInput {
            description: None,
            template_id,
            template_settings_id: 0,
            propose_settings,
            template_settings,
            scheduler_msg: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct Proposal {
    pub desc: u32,
    pub created: TimestampSec,
    pub created_by: AccountId,
    pub end: TimestampSec,
    pub votes: HashMap<AccountId, u8>,
    pub state: ProposalState,
    pub workflow_id: u16,
    pub workflow_settings_id: u8,
    pub voting_results: Vec<U128>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum ProposalState {
    InProgress,
    Invalid,
    Spam,
    Rejected,
    Accepted,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum VersionedProposal {
    V1(Proposal),
}

impl From<VersionedProposal> for Proposal {
    fn from(v: VersionedProposal) -> Self {
        match v {
            VersionedProposal::V1(v) => v,
        }
    }
}
