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

#[derive(Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq, Serialize))]
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn tags_crud_scenario() {
        let new_input: Vec<String> = vec!["tag1".into(), "tag2".into(), "tag3".into()];
        let mut tags = Tags::new();
        let result = tags.insert(new_input);

        assert_eq!(result, Some((1, 3)));

        let mut expected_hm = HashMap::new();
        expected_hm.insert(1, "tag1".into());
        expected_hm.insert(2, "tag2".into());
        expected_hm.insert(3, "tag3".into());
        let expected_tags = Tags {
            last_id: 3,
            map: expected_hm.clone(),
        };

        assert_eq!(tags, expected_tags);

        let insert_input = vec!["tag4".into(), "tag5".into(), "tag6".into()];
        let result = tags.insert(insert_input);

        assert_eq!(result, Some((4, 6)));

        expected_hm.insert(4, "tag4".into());
        expected_hm.insert(5, "tag5".into());
        expected_hm.insert(6, "tag6".into());
        let expected_tags = Tags {
            last_id: 6,
            map: expected_hm.clone(),
        };

        assert_eq!(tags, expected_tags);

        tags.rename(2, "yolo tag".into());

        expected_hm.insert(2, "yolo tag".into());
        let expected_tags = Tags {
            last_id: 6,
            map: expected_hm.clone(),
        };

        assert_eq!(tags, expected_tags);

        tags.remove(1);
        expected_hm.remove(&1);

        let expected_tags = Tags {
            last_id: 6,
            map: expected_hm,
        };

        assert_eq!(tags, expected_tags);
        assert_eq!(tags.map.len(), expected_tags.map.len());
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
