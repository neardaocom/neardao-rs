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

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
#[repr(transparent)]
pub struct GroupMembers(HashMap<AccountId, Vec<TagId>>);
impl GroupMembers {
    /// Adds members to group.
    /// Overrides existing.
    /// Returns false if the member was not in the group before.
    pub fn add_member(&mut self, member: GroupMember) -> bool {
        self.0.insert(member.account_id, member.tags).is_some()
    }
    pub fn remove_member(&mut self, account_id: AccountId) -> Option<GroupMember> {
        self.0
            .remove(&account_id)
            .map(|tags| GroupMember { account_id, tags })
    }
    pub fn members_count(&self) -> usize {
        self.0.len()
    }
    pub fn get_members(&self) -> Vec<GroupMember> {
        self.0
            .iter()
            .map(|(a, t)| GroupMember {
                account_id: a.clone(),
                tags: t.clone(),
            })
            .collect()
    }
}

impl From<Vec<GroupMember>> for GroupMembers {
    fn from(input: Vec<GroupMember>) -> Self {
        let mut members = HashMap::with_capacity(input.len());
        for i in input.into_iter() {
            members.insert(i.account_id, i.tags);
        }
        GroupMembers(members)
    }
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
    pub members: GroupMembers,
}

impl Group {
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
        Group {
            settings,
            members: members.into(),
        }
    }

    /// Adds members to the group.
    /// Returns count of new members added.
    /// New added + overwriten = `members.len()`
    pub fn add_members(&mut self, members: Vec<GroupMember>) -> u32 {
        let mut new_added = 0;
        for m in members.into_iter() {
            if !self.members.add_member(m) {
                new_added += 1;
            }
        }
        new_added
    }

    /// Removes member from group.
    /// If the member is leader, then group leader is removed from it's settings.
    pub fn remove_member(&mut self, account_id: AccountId) -> Option<GroupMember> {
        if let Some(ref leader) = self.settings.leader {
            if account_id == *leader {
                self.settings.leader = None;
            }
        }
        self.members.remove_member(account_id)
    }

    pub fn get_members_accounts(&self) -> Vec<AccountId> {
        self.members
            .get_members()
            .into_iter()
            .map(|member| member.account_id)
            .collect()
    }

    pub fn get_member_by_account(&self, account_id: &AccountId) -> Option<GroupMember> {
        self.members
            .get_members()
            .into_iter()
            .find(|m| m.account_id == *account_id)
    }

    // TODO: This is not true, tags does not mean role.
    pub fn get_members_accounts_by_role(&self, role: TagId) -> Vec<AccountId> {
        self.members
            .get_members()
            .into_iter()
            .filter(|m| m.tags.iter().any(|r| *r == role))
            .map(|m| m.account_id)
            .collect()
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
    pub fn add_group(&mut self, group: GroupInput) {
        self.group_last_id += 1;
        let mut cache = HashMap::with_capacity(group.members.len());
        for member in group.members.iter() {
            let mut user_roles = if let Some(roles) = self.user_roles.get(&member.account_id) {
                roles
            } else {
                self.total_members_count += 1;
                UserRoles::default()
            };
            user_roles.add_new_group(self.group_last_id);
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
        for (account, roles) in cache.into_iter() {
            self.user_roles.insert(account, &roles);
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

    /// TODO: Refactor.
    pub fn group_remove(&mut self, id: GroupId) -> bool {
        if let Some(mut group) = self.groups.remove(&id) {
            //let token_lock: TokenLock = group.remove_storage_data();
            //self.ft_total_locked -= token_lock.amount - token_lock.distributed;
            self.total_members_count -= group.members.members_count() as u32;
            true
        } else {
            false
        }
    }

    pub fn group_update(&mut self, id: GroupId, settings: GroupSettings) -> bool {
        if let Some(mut group) = self.groups.get(&id) {
            group.settings = settings;
            self.groups.insert(&id, &group);
            true
        } else {
            false
        }
    }

    pub fn group_add_members(&mut self, id: GroupId, members: Vec<GroupMember>) -> bool {
        if let Some(mut group) = self.groups.get(&id) {
            self.total_members_count += group.add_members(members);
            self.groups.insert(&id, &group);
            true
        } else {
            false
        }
    }

    pub fn group_remove_member(&mut self, id: GroupId, member: AccountId) -> bool {
        if let Some(mut group) = self.groups.get(&id) {
            group.remove_member(member);
            self.total_members_count -= 1;
            self.groups.insert(&id, &group);
            true
        } else {
            false
        }
    }
}
