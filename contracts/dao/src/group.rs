use std::collections::HashMap;

use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
    AccountId, IntoStorageKey,
};

use crate::{GroupId, TagId};

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
    pub name: String,
    pub leader: Option<AccountId>,
    pub parent_group: GroupId,
}

#[derive(Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq, Serialize))]
#[serde(crate = "near_sdk::serde")]
pub struct GroupInput {
    pub settings: GroupSettings,
    pub members: Vec<GroupMember>,
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct GroupOutput {
    pub id: GroupId,
    pub settings: GroupSettings,
    pub members: Vec<GroupMember>,
}

impl GroupOutput {
    pub fn from_group(id: GroupId, group: Group) -> Self {
        GroupOutput {
            id,
            settings: group.settings,
            members: group.members.get_members(),
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
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
                "Leader must be contained in group members"
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

    pub fn get_members_accounts_by_role(&self, role: TagId) -> Vec<AccountId> {
        self.members
            .get_members()
            .into_iter()
            .filter(|m| m.tags.iter().any(|r| *r == role))
            .map(|m| m.account_id)
            .collect()
    }
}
