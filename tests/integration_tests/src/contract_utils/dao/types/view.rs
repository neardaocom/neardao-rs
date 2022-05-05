use library::{
    types::datatype::Value,
    workflow::{
        instance::Instance,
        settings::{ProposeSettings, TemplateSettings},
        template::Template,
    },
};

use super::proposal::VProposal;

pub(crate) type ViewInstance = Option<(Instance, ProposeSettings)>;
pub(crate) type ViewTemplates = Vec<(u16, (Template, Vec<TemplateSettings>))>;
pub(crate) type ViewProposal = Option<(VProposal, Option<Vec<TemplateSettings>>)>;
pub(crate) type ViewWorkflowStorage = Option<Vec<(String, Value)>>;
