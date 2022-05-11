//! Workflow provider contract
//! Providers workflow templates with necessary object metadata for them to work.

#![allow(unused_imports)]
use library::workflow::help::TemplateHelp;
use library::workflow::settings::TemplateSettings;
use library::workflow::template::Template;
use library::workflow::types::ObjectMetadata;
use library::{FnCallId, MethodName, Version};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap};
use near_sdk::env;
use near_sdk::serde::Serialize;
use near_sdk::{near_bindgen, BorshStorageKey};

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    WorkflowTemplate,
    WorkflowHelp,
    FnCallMetadata,
    StandardFnCallMetadata,
    WorkflowFnCalls,
    WfAddSettings,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    // Id 0 is reserved for "wf_add".
    last_wf_id: u16,
    workflows: UnorderedMap<u16, Template>,
    workflow_help: UnorderedMap<u16, TemplateHelp>,
    /// Function calls for workflow template.
    workflow_fncalls: LookupMap<u16, (Vec<FnCallId>, Vec<MethodName>)>,
    /// Object metadata for fn_call id.
    fncall_metadata: UnorderedMap<FnCallId, Vec<ObjectMetadata>>,
    standard_fncall_metadata: UnorderedMap<MethodName, Vec<ObjectMetadata>>,
    wf_add_settings: LazyOption<Vec<TemplateSettings>>,
}

#[near_bindgen]
impl Contract {
    #[private]
    #[allow(unused_mut)]
    pub fn workflow_add(
        &mut self,
        workflow: Template,
        fncalls: Vec<FnCallId>,
        mut fncall_metadata: Vec<Vec<ObjectMetadata>>,
        help: Option<TemplateHelp>,
    ) {
        if self.last_wf_id == 0 {
            assert!(
                workflow.code.contains("wf_add"),
                "0th template must be valid wf_add"
            );
        }
        assert_eq!(fncalls.len(), fncall_metadata.len());
        self.workflows.insert(&self.last_wf_id, &workflow);
        for fncall in fncalls.iter().rev() {
            self.fncall_metadata
                .insert(fncall, &(fncall_metadata.pop().unwrap()));
        }
        if let Some(help) = help {
            self.workflow_help.insert(&self.last_wf_id, &help);
        }
        self.workflow_fncalls
            .insert(&self.last_wf_id, &(fncalls, vec![]));
        self.last_wf_id += 1;
    }

    /// Adds fncalls for standard interfaces. Eg. FT Standard NEP-148.
    #[private]
    #[allow(unused_mut)]
    pub fn standard_fncalls_add(
        &mut self,
        fncalls: Vec<MethodName>,
        mut fncall_metadata: Vec<Vec<ObjectMetadata>>,
    ) {
        assert_eq!(fncalls.len(), fncall_metadata.len());

        for fncall in fncalls.iter().rev() {
            self.standard_fncall_metadata
                .insert(fncall, &(fncall_metadata.pop().unwrap()));
        }
    }

    #[private]
    pub fn workflow_add_help(&mut self, id: u16, wf_help: TemplateHelp) -> Option<TemplateHelp> {
        self.workflow_help.insert(&id, &wf_help)
    }

    #[private]
    pub fn workflow_remove(&mut self, id: u16) -> bool {
        self.workflow_fncalls.remove(&id);
        self.workflows.remove(&id).is_some()
    }

    /// Returns Workflow with corresponding FnCalls and their metadata
    #[allow(clippy::type_complexity)]
    pub fn wf_template(
        self,
        id: u16,
    ) -> Option<(Template, Vec<FnCallId>, Vec<Vec<ObjectMetadata>>)> {
        match self.workflows.get(&id) {
            Some(t) => match self.workflow_fncalls.get(&id) {
                Some((fncalls, _)) => {
                    let mut fncalls_metadata = Vec::with_capacity(fncalls.len());
                    for fncall in fncalls.iter() {
                        fncalls_metadata.push(self.fncall_metadata.get(fncall).unwrap());
                    }
                    Some((t, fncalls, fncalls_metadata))
                }
                None => panic!("Missing FnCalls for the required template."),
            },
            _ => None,
        }
    }

    pub fn wf_templates(self) -> Vec<Metadata> {
        self.workflows
            .to_vec()
            .into_iter()
            .map(|(id, t)| {
                let (fncalls, standard_fncalls) = self.workflow_fncalls.get(&id).unwrap();
                let help = self.workflow_help.get(&id).is_some();
                Metadata {
                    id,
                    code: t.code,
                    version: t.version,
                    fncalls,
                    standard_fncalls,
                    help,
                }
            })
            .collect()
    }

    pub fn wf_conditions_help(self, id: u16) -> Option<TemplateHelp> {
        self.workflow_help.get(&id)
    }

    pub fn wf_template_fncalls(self, id: u16) -> Vec<FnCallId> {
        self.workflow_fncalls
            .get(&id)
            .map(|(fns, _)| fns)
            .unwrap_or_else(Vec::new)
    }

    pub fn wf_template_standard_fncalls(self, id: u16) -> Vec<MethodName> {
        self.workflow_fncalls
            .get(&id)
            .map(|(_, fns)| fns)
            .unwrap_or_else(Vec::new)
    }

    pub fn standard_fn_call_metadata(self, method: String) -> Vec<ObjectMetadata> {
        self.standard_fncall_metadata
            .get(&method)
            .unwrap_or_else(Vec::new)
    }

    pub fn fncall_metadata(self, id: FnCallId) -> Vec<ObjectMetadata> {
        self.fncall_metadata.get(&id).unwrap_or_else(Vec::new)
    }

    pub fn standard_fncalls(self) -> Vec<(MethodName, Vec<ObjectMetadata>)> {
        self.standard_fncall_metadata.iter().collect()
    }

    /// Returns workflow add with its default template settings.
    pub fn default_wf_add(
        self,
    ) -> (
        Template,
        FnCallId,
        Vec<ObjectMetadata>,
        Vec<TemplateSettings>,
    ) {
        let wf = self.workflows.get(&0).unwrap();
        let fncall = self.workflow_fncalls.get(&0).unwrap().0.remove(0);
        let fncalls_metadata = self.fncall_metadata.get(&fncall).unwrap();
        let settings = self.wf_add_settings.get().unwrap();
        (wf, fncall, fncalls_metadata, settings)
    }

    #[private]
    pub fn add_wf_add_settings(&mut self, settings: TemplateSettings) {
        let mut wf_add_settings = self.wf_add_settings.get().unwrap();
        wf_add_settings.push(settings);
        self.wf_add_settings.set(&wf_add_settings);
    }
    #[private]
    pub fn add_wf_remove_settings(&mut self, id: usize) {
        let mut wf_add_settings = self.wf_add_settings.get().unwrap();
        wf_add_settings.remove(id);
        self.wf_add_settings.set(&wf_add_settings);
    }
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            last_wf_id: 0,
            workflows: UnorderedMap::new(StorageKeys::WorkflowTemplate),
            workflow_help: UnorderedMap::new(StorageKeys::WorkflowHelp),
            fncall_metadata: UnorderedMap::new(StorageKeys::FnCallMetadata),
            standard_fncall_metadata: UnorderedMap::new(StorageKeys::StandardFnCallMetadata),
            workflow_fncalls: LookupMap::new(StorageKeys::WorkflowFnCalls),
            wf_add_settings: LazyOption::new(StorageKeys::WorkflowFnCalls, Some(&vec![])),
        }
    }
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Metadata {
    pub id: u16,
    pub code: String,
    pub version: Version,
    pub fncalls: Vec<FnCallId>,
    pub standard_fncalls: Vec<MethodName>,
    pub help: bool,
}
