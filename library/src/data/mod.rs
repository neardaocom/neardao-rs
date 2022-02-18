#![cfg(test)]
#![allow(unused)]
use crate::{
    types::FnCallMetadata,
    workflow::{ProposeSettings, Template, TemplateSettings},
    FnCallId,
};

// Workflow data for tests and provider
pub mod basic_workflows;
pub mod bounty;
pub mod skyward;

// Output for provider
pub mod output;

pub type TemplateData = (Template, Vec<FnCallId>, Vec<Vec<FnCallMetadata>>);
pub type TemplateUserSettings = (Vec<TemplateSettings>, ProposeSettings);
