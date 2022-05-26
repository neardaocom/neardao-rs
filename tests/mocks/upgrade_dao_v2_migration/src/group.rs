use std::collections::HashMap;

use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
    AccountId,
};

use crate::{
    core::Contract,
    internal::utils::current_timestamp_sec,
    reward::Reward,
    role::{MemberRoles, Roles, UserRoles},
    GroupId, RewardId, RoleId, TagId,
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

impl GroupMember {
    pub fn new(account_id: AccountId) -> Self {
        Self {
            account_id,
            tags: vec![],
        }
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

#[derive(Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct GroupInput {
    /// Group settings.
    pub settings: GroupSettings,
    /// Collection of group member account and its tag ids.
    /// Each member get default group role.
    pub members: Vec<GroupMember>,
    /// Map of additional roles and for provided accounts.
    /// All accounts must be included in `members`.
    pub member_roles: Vec<MemberRoles>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Group {
    pub settings: GroupSettings,
    pub members: HashMap<AccountId, Vec<TagId>>,
    /// Reward ids of all related rewards.
    pub rewards: Vec<(RewardId, RoleId)>,
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
            rewards: vec![],
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
    pub fn group_reward_ids(&self) -> Vec<(RewardId, RoleId)> {
        self.rewards.to_owned()
    }
    pub fn add_new_reward(&mut self, reward_id: u16, role_id: u16) {
        self.rewards.push((reward_id, role_id))
    }
}
