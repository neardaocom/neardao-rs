use library::{
    workflow::{
        settings::{ProposeSettings, TemplateSettings},
        template::Template,
        types::ObjectMetadata,
    },
    FnCallId, MethodName,
};

pub mod object_metadata;
pub mod workflow;

#[cfg(test)]
pub mod output;

pub type TemplateData = (Template, Vec<FnCallId>, Vec<Vec<ObjectMetadata>>, Vec<MethodName>);
pub type TemplateUserSettings = (Vec<TemplateSettings>, ProposeSettings);
