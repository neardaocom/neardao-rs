//! Workflow provider contract
//! Provide workflow templates with necessary object metadata for them to work.

#![allow(unused_mut)]
use library::workflow::settings::TemplateSettings;
use library::workflow::template::Template;
use library::workflow::types::ObjectMetadata;
use library::{FnCallId, MethodName, Version};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap};
use near_sdk::serde::Serialize;
use near_sdk::{near_bindgen, BorshStorageKey};

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    WorkflowTemplate,
    FnCallMetadata,
    StandardFnCallMetadata,
    WorkflowFnCalls,
    WfAddSettings,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    // Id 0 is reserved for "basic_pkg".
    last_wf_id: u16,
    workflows: UnorderedMap<u16, Template>,
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
    pub fn workflow_add(
        &mut self,
        workflow: Template,
        fncalls: Vec<FnCallId>,
        standard_fncalls: Vec<MethodName>,
        mut fncall_metadata: Vec<Vec<ObjectMetadata>>,
    ) {
        if self.last_wf_id == 0 {
            assert!(
                workflow.code.contains("basic_pkg"),
                "0th template must be valid basic_pkg*"
            );
        }
        assert_eq!(fncalls.len(), fncall_metadata.len());
        self.workflows.insert(&self.last_wf_id, &workflow);
        for fncall in fncalls.iter().rev() {
            self.fncall_metadata
                .insert(fncall, &(fncall_metadata.pop().unwrap()));
        }
        self.workflow_fncalls
            .insert(&self.last_wf_id, &(fncalls, standard_fncalls));
        self.last_wf_id += 1;
    }

    /// Adds fncalls for standard interfaces. Eg. FT Standard NEP-148.
    #[private]
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
    pub fn workflow_remove(&mut self, id: u16) -> bool {
        self.workflow_fncalls.remove(&id);
        self.workflows.remove(&id).is_some()
    }

    /// Returns Workflow with corresponding FnCalls and their metadata.
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
                Metadata {
                    id,
                    code: t.code,
                    version: t.version,
                    fncalls,
                    standard_fncalls,
                }
            })
            .collect()
    }

    pub fn wf_template_fncalls(self, id: u16) -> Vec<FnCallId> {
        self.workflow_fncalls
            .get(&id)
            .map(|(fns, _)| fns)
            .unwrap_or_default()
    }

    pub fn wf_template_standard_fncalls(self, id: u16) -> Vec<MethodName> {
        self.workflow_fncalls
            .get(&id)
            .map(|(_, fns)| fns)
            .unwrap_or_default()
    }

    pub fn standard_fn_call_metadata(self, method: String) -> Vec<ObjectMetadata> {
        self.standard_fncall_metadata
            .get(&method)
            .unwrap_or_default()
    }

    pub fn fncall_metadata(self, id: FnCallId) -> Vec<ObjectMetadata> {
        self.fncall_metadata.get(&id).unwrap_or_default()
    }

    pub fn standard_fncalls(self) -> Vec<(MethodName, Vec<ObjectMetadata>)> {
        self.standard_fncall_metadata.iter().collect()
    }

    /// Return basic workflow package with its default template settings.
    pub fn wf_basic_package(
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
    pub fn wf_basic_package_add_settings(&mut self, settings: TemplateSettings) {
        let mut wf_add_settings = self.wf_add_settings.get().unwrap();
        wf_add_settings.push(settings);
        self.wf_add_settings.set(&wf_add_settings);
    }
    #[private]
    pub fn wf_basic_package_remove_settings(&mut self, id: usize) {
        let mut wf_add_settings = self.wf_add_settings.get().unwrap();
        wf_add_settings.remove(id);
        self.wf_add_settings.set(&wf_add_settings);
    }
    #[private]
    pub fn clear_state(&mut self) {
        self.workflows.clear();
        for i in 0..self.last_wf_id {
            self.workflow_fncalls.remove(&i);
        }
        self.last_wf_id = 0;
        self.fncall_metadata.clear();
        self.standard_fncall_metadata.clear();
        self.wf_add_settings.remove();
    }
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            last_wf_id: 0,
            workflows: UnorderedMap::new(StorageKeys::WorkflowTemplate),
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
}
