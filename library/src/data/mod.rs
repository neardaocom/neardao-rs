#![allow(unused)]

pub mod standard_fn_calls;

// Workflow data for tests and provider
pub mod basic_workflows;
//pub mod bounty;
pub mod skyward;

use crate::{
    workflow::{
        settings::{ProposeSettings, TemplateSettings},
        template::Template,
        types::ObjectMetadata,
    },
    FnCallId,
};

// Output for provider
#[cfg(test)]
pub mod output;

pub type TemplateData = (Template, Vec<FnCallId>, Vec<Vec<ObjectMetadata>>);
pub type TemplateUserSettings = (Vec<TemplateSettings>, ProposeSettings);
