use library::workflow::settings::{ProposeSettings, TemplateSettings};
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ProposalType {
    WorkflowAdd(WorkflowAddOptions),
    Skyward(SkywardOptions),
}

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
