use library::workflow::Template;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::serde::Serialize;
use near_sdk::{env, near_bindgen, BorshStorageKey};

mod test;

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    WorkflowTemplate,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    last_wf_id: u16,
    workflows: UnorderedMap<u16, Template>,
}

#[near_bindgen]
impl Contract {
    #[private]
    pub fn workflow_add(&mut self, workflow: Template) {
        self.last_wf_id += 1;
        self.workflows.insert(&self.last_wf_id, &workflow);
    }

    #[private]
    pub fn workflow_remove(&mut self, id: u16) -> bool {
        self.workflows.remove(&id).map_or(false, |_| true)
    }

    pub fn get(self, id: u16) -> Option<Template> {
        self.workflows.get(&id)
    }

    pub fn list(self) -> Vec<Metadata> {
        self.workflows
            .to_vec()
            .into_iter()
            .map(|(id, t)| Metadata {
                id,
                name: t.name,
                version: t.version,
            })
            .collect()
    }
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            last_wf_id: 0,
            workflows: UnorderedMap::new(StorageKeys::WorkflowTemplate),
        }
    }
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Metadata {
    pub id: u16,
    pub name: String,
    pub version: u8,
}
