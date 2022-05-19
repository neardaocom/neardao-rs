use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use workspaces::AccountId;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct GroupMembers(pub HashMap<AccountId, Vec<u16>>);

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Group {
    pub settings: GroupSettings,
    pub members: GroupMembers,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct GroupInput {
    pub settings: GroupSettings,
    pub members: Vec<GroupMember>,
    pub member_roles: HashMap<String, Vec<String>>,
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct GroupSettings {
    pub name: String,
    pub leader: Option<AccountId>,
    pub parent_group: u16,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct GroupMember {
    pub account_id: AccountId,
    pub tags: Vec<u16>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Roles {
    last_id: u16,
    map: HashMap<u16, String>,
}
impl Roles {
    pub fn new() -> Self {
        Self {
            last_id: 0,
            map: HashMap::new(),
        }
    }
    pub fn add_role(mut self, name: &str) -> Self {
        self.last_id += 1;
        self.map.insert(self.last_id, name.to_string());
        Self {
            last_id: self.last_id,
            map: self.map,
        }
    }
    pub fn remove_role(mut self, name: &str) -> Self {
        if let Some(key) =
            self.map
                .clone()
                .into_iter()
                .find_map(|(key, val)| if val == name { Some(key) } else { None })
        {
            self.map.remove(&key);
        }
        Self {
            last_id: self.last_id,
            map: self.map,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct UserRoles(HashMap<u16, Vec<u16>>);

impl UserRoles {
    pub fn new() -> Self {
        Self(HashMap::new())
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
        }
        self
    }
    pub fn remove_role(mut self, role_id: u16) -> Self {
        self.0.remove(&role_id);
        self
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
}
