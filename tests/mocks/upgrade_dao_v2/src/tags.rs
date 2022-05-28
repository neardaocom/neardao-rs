//! Tags for DAO
//! Simple storage for strings which are then referenced by integer key.
//! Last inserted key is kept to avoid assigning same key to two different tags during the tags lifetime in DAO.
//! Contains no logic exepct basic CRUD methods.

use std::collections::{hash_map::Iter, HashMap};

use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
};

use crate::{core::Contract, TagId};

#[derive(Deserialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct TagInput {
    pub category: String,
    pub values: Vec<String>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Tags {
    last_id: u16,
    map: HashMap<TagId, String>,
}

impl Tags {
    pub fn new() -> Self {
        Tags {
            last_id: 0,
            map: HashMap::new(),
        }
    }

    /// Inserts new tags and returns tuple with new first id and last id.
    /// Does NOT check for duplicates - It's caller's responsibility not to insert already existing tags
    pub fn insert(&mut self, new_tags: Vec<String>) -> Option<(u16, u16)> {
        if new_tags.is_empty() {
            return None;
        }

        let ids = (self.last_id + 1, self.last_id + (new_tags.len() as u16));
        for s in new_tags.into_iter() {
            self.last_id += 1;
            self.map.insert(self.last_id, s);
        }

        Some(ids)
    }

    pub fn remove(&mut self, id: TagId) {
        self.map.remove(&id);
    }

    pub fn rename(&mut self, id: TagId, name: String) {
        if let Some(t) = self.map.get_mut(&id) {
            *t = name;
        }
    }

    pub fn get(&self, id: TagId) -> Option<&String> {
        self.map.get(&id)
    }

    pub fn iter(&self) -> Iter<TagId, String> {
        self.map.iter()
    }
}

impl Default for Tags {
    fn default() -> Self {
        Self::new()
    }
}

impl Contract {
    pub fn tag_add(&mut self, category: String, tags: Vec<String>) -> Option<(TagId, TagId)> {
        let mut t = self.tags.get(&category).unwrap_or_else(Tags::new);
        let ids = t.insert(tags);
        self.tags.insert(&category, &t);
        ids
    }

    pub fn tag_edit(&mut self, category: String, id: u16, value: String) -> bool {
        match self.tags.get(&category) {
            Some(mut t) => {
                t.rename(id, value);
                self.tags.insert(&category, &t);
                true
            }
            None => false,
        }
    }

    pub fn tag_remove(&mut self, category: String, id: u16) -> bool {
        match self.tags.get(&category) {
            Some(mut t) => {
                t.remove(id);
                self.tags.insert(&category, &t);
                true
            }
            None => false,
        }
    }
}
