use std::collections::{hash_map::Iter, HashMap};

use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::Serialize,
    AccountId,
};

use crate::{core::Contract, GroupId, RoleId};

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Roles {
    last_id: RoleId,
    map: HashMap<RoleId, String>,
}

impl Roles {
    pub fn new() -> Self {
        Roles {
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
    pub fn find_role_by_name(&self, name: &str) -> Option<RoleId> {
        self.map.iter().find(|(_, v)| **v == name).map(|e| *e.0)
    }
    pub fn find_role_by_id(&self, id: RoleId) -> Option<&str> {
        self.map
            .iter()
            .find(|(k, _)| **k == id)
            .map(|e| e.1.as_str())
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Default)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct UserRoles(HashMap<GroupId, Vec<RoleId>>);
impl UserRoles {
    pub fn add_group_role(&mut self, group_id: GroupId, role_id: RoleId) {
        if let Some(roles) = self.0.get_mut(&group_id) {
            if !roles.contains(&role_id) {
                roles.push(role_id);
            }
        } else {
            let roles = vec![role_id];
            self.0.insert(group_id, roles);
        }
    }
    pub fn remove_group_role(&mut self, group_id: GroupId, role_id: RoleId) {
        if let Some(roles) = self.0.get_mut(&group_id) {
            if let Some(pos) = roles.iter().position(|el| *el == role_id) {
                roles.swap_remove(pos);
            }
        }
    }
    pub fn remove_all_group_roles(&mut self, group_id: GroupId) {
        self.0.remove(&group_id);
    }
    pub fn has_group_role(&self, group_id: GroupId, role_id: RoleId) -> bool {
        if let Some(roles) = self.0.get(&group_id) {
            roles.contains(&role_id)
        } else {
            false
        }
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Contract {
    pub fn save_user_roles(&mut self, account_id: &AccountId, roles: &UserRoles) {
        if roles.is_empty() {
            self.user_roles.remove(account_id);
        } else if self.user_roles.insert(account_id, roles).is_none() {
            self.total_members_count += 1;
        }
    }
    pub fn remove_user_role_group(&mut self, account_id: &AccountId, group_id: u16) {
        if let Some(mut roles) = self.user_roles.get(&account_id) {
            roles.remove_all_group_roles(group_id);
            if !roles.is_empty() {
                self.user_roles.insert(account_id, &roles);
            } else {
                self.user_roles.remove(account_id);
            }
        }
    }
}
