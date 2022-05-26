use std::collections::{hash_map::Iter, HashMap};

use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
    AccountId,
};

use crate::{core::Contract, GroupId, RoleId};

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq, Clone))]
#[serde(crate = "near_sdk::serde")]
pub struct Roles {
    last_id: RoleId,
    map: HashMap<RoleId, String>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Default)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq, Clone))]
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

#[cfg(test)]
impl UserRoles {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
    pub fn sort(self) -> Self {
        let roles = self
            .0
            .into_iter()
            .map(|(k, mut v)| {
                v.sort();
                (k, v)
            })
            .collect();
        Self(roles)
    }
    pub fn add_group_roles(mut self, group_id: u16, roles: Vec<u16>) -> Self {
        self.0.insert(group_id, roles);
        self
    }
    pub fn add_role(mut self, group_id: u16, role_id: u16) -> Self {
        if let Some(roles) = self.0.get_mut(&group_id) {
            if !roles.contains(&role_id) {
                roles.push(role_id)
            }
        } else {
            self.0.insert(group_id, vec![role_id]);
        }
        self
    }
    pub fn remove_role(mut self, group_id: u16, role_id: u16) -> Self {
        if let Some(roles) = self.0.get_mut(&group_id) {
            if let Some(pos) = roles.iter().position(|el| *el == role_id) {
                roles.swap_remove(pos);
            }
        }
        self
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq, Clone, Default))]
#[serde(crate = "near_sdk::serde")]
pub struct MemberRoles {
    pub name: String,
    pub members: Vec<AccountId>,
}

impl Contract {
    /// Save `roles` for `account_id`.
    /// If `roles` are empty, remove it from the contract.
    pub fn save_user_roles(&mut self, account_id: &AccountId, roles: &UserRoles) {
        if roles.is_empty() {
            self.user_roles.remove(account_id);
        } else if self.user_roles.insert(account_id, roles).is_none() {
            self.total_members_count += 1;
        }
    }
    /// Remove all group roles for `account_id`.
    /// Save if not empty.
    /// Remove it from the contract othewise.
    pub fn remove_user_role_group(&mut self, account_id: &AccountId, group_id: u16) {
        if let Some(mut roles) = self.user_roles.get(&account_id) {
            roles.remove_all_group_roles(group_id);
            if !roles.is_empty() {
                self.user_roles.insert(account_id, &roles);
            } else {
                self.user_roles.remove(account_id);
                self.total_members_count -= 1;
            }
        }
    }
}
