use library::{
    workflow::{
        settings::{ProposeSettings, TemplateSettings},
        template::Template,
        types::ObjectMetadata,
    },
    FnCallId,
};

pub mod object_metadata;
pub mod workflow;

#[cfg(test)]
pub mod output;

pub type TemplateData = (Template, Vec<FnCallId>, Vec<Vec<ObjectMetadata>>);
pub type TemplateUserSettings = (Vec<TemplateSettings>, ProposeSettings);
