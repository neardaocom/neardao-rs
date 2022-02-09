use library::types::FnCallMetadata;
use library::workflow::Template;
use library::FnCallId;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap};
use near_sdk::serde::Serialize;
use near_sdk::{env, near_bindgen, BorshStorageKey};

mod test;

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    WorkflowTemplate,
    FnCallMetadata,
    WorkflowFnCalls,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    last_wf_id: u16,
    workflows: UnorderedMap<u16, Template>,
    workflow_fncalls: LookupMap<u16, Vec<FnCallId>>,
    fncall_metadata: UnorderedMap<FnCallId, Vec<FnCallMetadata>>,
}

#[near_bindgen]
impl Contract {
    #[private]
    pub fn workflow_add(
        &mut self,
        workflow: Template,
        fncalls: Vec<FnCallId>,
        mut fncall_metadata: Vec<Vec<FnCallMetadata>>,
    ) {
        assert_eq!(fncalls.len(), fncall_metadata.len());
        self.last_wf_id += 1;
        self.workflows.insert(&self.last_wf_id, &workflow);

        for fncall in fncalls.iter().rev() {
            self.fncall_metadata
                .insert(&fncall, &fncall_metadata.pop().unwrap());
        }

        self.workflow_fncalls.insert(&self.last_wf_id, &fncalls);
    }

    #[private]
    pub fn workflow_remove(&mut self, id: u16) -> bool {
        self.workflow_fncalls.remove(&id);
        self.workflows.remove(&id).is_some()
    }

    /// Returns Workflow with corresponding FnCalls and their metadata
    pub fn get(self, id: u16) -> Option<(Template, Vec<FnCallId>, Vec<Vec<FnCallMetadata>>)> {
        match self.workflows.get(&id) {
            Some(t) => match self.workflow_fncalls.get(&id) {
                Some(fncalls) => {
                    let mut fncalls_metadata = Vec::with_capacity(fncalls.len());
                    for fncall in fncalls.iter() {
                        fncalls_metadata.push(self.fncall_metadata.get(fncall).unwrap());
                    }
                    Some((t, fncalls, fncalls_metadata))
                }
                None => Some((t, vec![], vec![])),
            },
            _ => None,
        }
    }

    pub fn wf_templates(self) -> Vec<Metadata> {
        self.workflows
            .to_vec()
            .into_iter()
            .map(|(id, t)| {
                let fncalls = self.workflow_fncalls.get(&id).unwrap();
                Metadata {
                    id,
                    name: t.name,
                    version: t.version,
                    fncalls,
                }
            })
            .collect()
    }

    pub fn wf_template_fncalls(self, id: u16) -> Vec<FnCallId> {
        self.workflow_fncalls.get(&id).unwrap_or(vec![])
    }

    pub fn fncall_metadata(self, id: FnCallId) -> Vec<FnCallMetadata> {
        self.fncall_metadata.get(&id).unwrap_or(vec![])
    }
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            last_wf_id: 0,
            workflows: UnorderedMap::new(StorageKeys::WorkflowTemplate),
            fncall_metadata: UnorderedMap::new(StorageKeys::FnCallMetadata),
            workflow_fncalls: LookupMap::new(StorageKeys::WorkflowFnCalls),
        }
    }
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Metadata {
    pub id: u16,
    pub name: String,
    pub version: u8,
    pub fncalls: Vec<FnCallId>,
}
