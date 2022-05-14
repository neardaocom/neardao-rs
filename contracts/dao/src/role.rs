use std::collections::{hash_map::Iter, HashMap};

use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::Serialize,
};

use crate::RoleId;

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Role {
    last_id: RoleId,
    map: HashMap<RoleId, String>,
}

impl Role {
    pub fn new() -> Self {
        Role {
            last_id: 0,
            map: HashMap::new(),
        }
    }

    /// Inserts new role and returns assigned `RoleId`
    /// Return `None` if `new_role` is empty string.
    pub fn insert(&mut self, new_role: String) -> Option<RoleId> {
        if new_role.is_empty() {
            return None;
        }
        self.last_id += 1;
        self.map.insert(self.last_id, new_role);

        Some(self.last_id)
    }

    pub fn remove(&mut self, id: RoleId) {
        self.map.remove(&id);
    }

    pub fn rename(&mut self, id: RoleId, name: String) {
        if let Some(t) = self.map.get_mut(&id) {
            *t = name;
        }
    }

    pub fn get(&self, id: RoleId) -> Option<&String> {
        self.map.get(&id)
    }

    pub fn iter(&self) -> Iter<RoleId, String> {
        self.map.iter()
    }
}
