//! Workflow provider contract
//! Providers workflow templates with necessary object metadata for them to work.

#![allow(unused_imports)]
use library::workflow::help::TemplateHelp;
use library::workflow::template::Template;
use library::workflow::types::ObjectMetadata;
use library::{FnCallId, MethodName, Version};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap};
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
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    last_wf_id: u16,
    workflows: UnorderedMap<u16, Template>,
    workflow_help: UnorderedMap<u16, TemplateHelp>,
    /// Function calls for workflow template.
    workflow_fncalls: LookupMap<u16, (Vec<FnCallId>, Vec<MethodName>)>,
    /// Object metadata for fn_call id.
    fncall_metadata: UnorderedMap<FnCallId, Vec<ObjectMetadata>>,
    standard_fncall_metadata: UnorderedMap<MethodName, Vec<ObjectMetadata>>,
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
        assert_eq!(fncalls.len(), fncall_metadata.len());
        self.last_wf_id += 1;
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
    ) -> Option<(
        Template,
        Vec<FnCallId>,
        Vec<Vec<ObjectMetadata>>,
        //Vec<MethodName>,
        //Vec<Vec<ObjectMetadata>>,
    )> {
        match self.workflows.get(&id) {
            Some(t) => match self.workflow_fncalls.get(&id) {
                Some((fncalls, std_fncalls)) => {
                    let mut fncalls_metadata = Vec::with_capacity(fncalls.len());
                    for fncall in fncalls.iter() {
                        fncalls_metadata.push(self.fncall_metadata.get(fncall).unwrap());
                    }

                    let mut std_fncalls_metadata = Vec::with_capacity(std_fncalls.len());
                    for std_fncall in std_fncalls.iter() {
                        std_fncalls_metadata
                            .push(self.standard_fncall_metadata.get(std_fncall).unwrap());
                    }
                    Some((
                        t,
                        fncalls,
                        fncalls_metadata,
                        //std_fncalls,
                        //std_fncalls_metadata,
                    ))
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
