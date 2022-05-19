use std::collections::HashMap;

use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
    AccountId,
};

use crate::{
    core::Contract,
    role::{Roles, UserRoles},
    GroupId, TagId,
};

#[derive(Serialize, Deserialize)]
#[cfg_attr(
    not(target_arch = "wasm32"),
    derive(Debug, PartialEq, Clone, PartialOrd, Ord, Eq)
)]
#[serde(crate = "near_sdk::serde")]
pub struct GroupMember {
    pub account_id: AccountId,
    pub tags: Vec<TagId>,
}

#[derive(Deserialize, BorshDeserialize, BorshSerialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct GroupSettings {
    /// Name of the group.
    pub name: String,
    /// Leader of the group.
    /// Must be included among provided group members.
    pub leader: Option<AccountId>,
    /// Reference to parent group.
    /// ATM its only evidence value.
    /// GroupId = 0 means no parent group.
    pub parent_group: GroupId,
}

#[derive(Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq, Serialize))]
#[serde(crate = "near_sdk::serde")]
pub struct GroupInput {
    /// Group settings.
    pub settings: GroupSettings,
    /// Collection of group member account and its tag ids.
    /// Each member get default group role.
    pub members: Vec<GroupMember>,
    /// Map of additional roles and for provided accounts.
    /// All accounts must be included in `members`.
    pub member_roles: HashMap<String, Vec<AccountId>>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Group {
    pub settings: GroupSettings,
    pub members: HashMap<AccountId, Vec<TagId>>,
}

impl Group {
    // TODO: Remove panic.
    pub fn new(settings: GroupSettings, members: Vec<GroupMember>) -> Self {
        if let Some(ref leader) = settings.leader {
            assert!(
                !leader.as_str().is_empty(),
                "Leader cannot be empty string."
            );
            assert!(
                members.iter().any(|m| *leader == m.account_id),
                "Leader must be contained in group members."
            );
        }
        let mut hm_members = HashMap::with_capacity(members.len());
        for i in members.into_iter() {
            hm_members.insert(i.account_id, i.tags);
        }
        Group {
            settings,
            members: hm_members,
        }
    }
    pub fn members_count(&self) -> usize {
        self.members.len()
    }

    /// Add member to the group.
    /// Return true if member was overwriten.
    pub fn add_member(&mut self, member: GroupMember) -> bool {
        self.members
            .insert(member.account_id, member.tags)
            .is_some()
    }

    /// Removes member from group.
    /// Return true if actually removed.
    /// If the member is leader, then group leader is removed from it's settings.
    pub fn remove_member(&mut self, account_id: &AccountId) -> bool {
        if let Some(ref leader) = self.settings.leader {
            if *account_id == *leader {
                self.settings.leader = None;
            }
        }
        self.members.remove(account_id).is_some()
    }
    pub fn get_members_accounts_refs(&self) -> Vec<&AccountId> {
        self.members.iter().map(|member| member.0).collect()
    }

    pub fn get_members_accounts(&self) -> Vec<AccountId> {
        self.members
            .clone()
            .into_iter()
            .map(|member| member.0)
            .collect()
    }
    pub fn is_member(&self, account_id: &AccountId) -> bool {
        self.members.contains_key(account_id)
    }
    pub fn is_account_id_leader(&self, account_id: &AccountId) -> bool {
        if let Some(ref leader) = self.settings.leader {
            *leader == *account_id
        } else {
            false
        }
    }
    pub fn group_leader(&self) -> Option<&AccountId> {
        self.settings.leader.as_ref()
    }
}

impl Contract {
    /// Add `group` to the contract.
    /// Also add defined roles to all group users.
    /// Update internal statistics.
    /// TODO: Refactoring maybe.
    pub fn group_add(&mut self, group: GroupInput) {
        self.group_last_id += 1;
        let mut cache = HashMap::with_capacity(group.members.len());
        for member in group.members.iter() {
            let user_roles = self
                .user_roles
                .get(&member.account_id)
                .unwrap_or_else(UserRoles::default);
            cache.insert(&member.account_id, user_roles);
        }
        let mut group_roles = Roles::new();
        for (role_name, members) in group.member_roles {
            if let Some(role_id) = group_roles.insert(role_name) {
                for member in members {
                    let user_roles = cache.get_mut(&member).expect("User roles do not match.");
                    user_roles.add_group_role(self.group_last_id, role_id);
                }
            }
        }
        for (account, mut roles) in cache.into_iter() {
            roles.add_group_role(self.group_last_id, 0);
            self.save_user_roles(account, &roles);
        }
        self.group_roles.insert(&self.group_last_id, &group_roles);
        self.groups.insert(
            &self.group_last_id,
            &Group::new(group.settings, group.members),
        );
    }

    /// TODO: Figure out better as its currently quite expensive solution.
    pub fn get_group_members_with_role(
        &self,
        group_id: u16,
        group: &Group,
        role_id: u16,
    ) -> Vec<AccountId> {
        let mut result_members = vec![];
        let group_members = group.get_members_accounts();
        for member in group_members {
            let member_roles = self.user_roles.get(&member);
            if let Some(roles) = member_roles {
                if roles.has_group_role(group_id, role_id) {
                    result_members.push(member);
                    break;
                }
            }
        }
        result_members
    }

    pub fn group_add_members(&mut self, id: GroupId, members: Vec<GroupMember>) -> bool {
        if let Some(mut group) = self.groups.get(&id) {
            for member in members.into_iter() {
                let mut user_roles = self
                    .user_roles
                    .get(&member.account_id)
                    .unwrap_or_else(UserRoles::default);
                user_roles.add_group_role(id, 0);
                self.save_user_roles(&member.account_id, &user_roles);
                group.add_member(member);
            }
            self.groups.insert(&id, &group);
            true
        } else {
            false
        }
    }

    pub fn group_remove_member(&mut self, id: GroupId, account_id: AccountId) -> bool {
        if let Some(mut group) = self.groups.get(&id) {
            if group.remove_member(&account_id) {
                self.remove_user_role_group(&account_id, id);
            };
            self.groups.insert(&id, &group);
            true
        } else {
            false
        }
    }

    pub fn group_remove(&mut self, id: GroupId) {
        if let Some(group) = self.groups.get(&id) {
            for account_id in group.get_members_accounts_refs() {
                self.remove_user_role_group(&account_id, id);
            }
            self.groups.remove(&id);
        }
    }
}
