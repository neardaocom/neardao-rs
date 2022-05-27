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
    pub fn remove_group_reward_id(&mut self, reward_id: RewardId) -> bool {
        if let Some(pos) = self.rewards.iter().position(|(r1, _)| *r1 == reward_id) {
            self.rewards.swap_remove(pos);
            true
        } else {
            false
        }
    }
    pub fn add_new_reward(&mut self, reward_id: u16, role_id: u16) {
        self.rewards.push((reward_id, role_id))
    }
}

impl Contract {
    /// Add `group` to the contract.
    /// Also add defined roles to all group users.
    /// Update internal statistics.
    /// TODO: Refactoring maybe.
    pub fn group_add(&mut self, group: GroupInput) {
        self.group_last_id += 1;
        self.internal_add_group_roles(
            self.group_last_id,
            group.members.as_slice(),
            group.member_roles,
        );
        self.groups.insert(
            &self.group_last_id,
            &Group::new(group.settings, group.members),
        );
    }

    pub fn get_group_members_with_role(
        &self,
        group_id: u16,
        group: &Group,
        role_id: u16,
    ) -> Vec<AccountId> {
        let group_members = group.get_members_accounts();
        if role_id == 0 {
            return group_members;
        }
        let mut result_members = vec![];
        for member in group_members {
            let member_roles = self.user_roles.get(&member);
            if let Some(roles) = member_roles {
                if roles.has_group_role(group_id, role_id) {
                    result_members.push(member);
                }
            }
        }
        result_members
    }

    /// Add members to the group if the group exist.
    /// It does following:
    /// - Add all `members` to the group, add them group role and assign all existing group member rewards.
    /// - If a member in the `members` has defined member role in `member_roles`,
    /// then he gets assigned the role and all existing role rewards.
    /// - Add all member roles to the group and to the members defined.
    /// NOTE: Accounts defined in `member_roles` but not in defined `members` nor in the existing group members,
    /// will be ignored.
    /// TODO: Refactor maybe.
    pub fn group_add_members(
        &mut self,
        id: GroupId,
        members: Vec<GroupMember>,
        member_roles: Vec<MemberRoles>,
    ) -> bool {
        if let Some(mut group) = self.groups.get(&id) {
            let mut account_roles_cache: HashMap<AccountId, Vec<u16>> = HashMap::new();
            if !member_roles.is_empty() {
                let mut group_roles = self
                    .group_roles
                    .get(&id)
                    .expect("internal - group roles not found");
                for role in member_roles {
                    let role_id =
                        if let Some(role_id) = group_roles.find_role_by_name(role.name.as_str()) {
                            role_id
                        } else {
                            group_roles.insert(role.name).expect("empty role name")
                        };
                    for account_id in role.members {
                        if let Some(roles) = account_roles_cache.get_mut(&account_id) {
                            roles.push(role_id);
                        } else {
                            account_roles_cache.insert(account_id, vec![role_id]);
                        }
                    }
                }
                self.group_roles.insert(&id, &group_roles);
            }
            let current_timestamp = current_timestamp_sec();
            let group_rewards = group.group_reward_ids();
            let rewards: Vec<(RewardId, Reward)> = group_rewards
                .into_iter()
                .map(|(r, _)| (r, self.rewards.get(&r).unwrap().into()))
                .collect();
            for member in members.into_iter() {
                let mut user_roles = self
                    .user_roles
                    .get(&member.account_id)
                    .unwrap_or_else(UserRoles::default);
                user_roles.add_group_role(id, 0);
                if let Some(roles) = account_roles_cache.remove(&member.account_id) {
                    for role_id in roles {
                        user_roles.add_group_role(id, role_id);
                    }
                }
                self.save_user_roles(&member.account_id, &user_roles);
                self.add_wallet_rewards(&member.account_id, rewards.clone(), current_timestamp);
                group.add_member(member);
            }
            for (account_id, roles) in account_roles_cache.into_iter() {
                if group.is_member(&account_id) {
                    let mut user_roles = self
                        .user_roles
                        .get(&account_id)
                        .expect("internal - user roles not found");
                    for role_id in roles {
                        user_roles.add_group_role(id, role_id);
                    }
                    self.save_user_roles(&account_id, &user_roles);
                    self.add_wallet_rewards(&account_id, rewards.clone(), current_timestamp);
                }
            }
            self.groups.insert(&id, &group);
            true
        } else {
            false
        }
    }
    pub fn group_remove_members(&mut self, id: GroupId, account_ids: Vec<AccountId>) -> bool {
        if let Some(mut group) = self.groups.get(&id) {
            let rewards: Vec<(u16, u16)> = group.group_reward_ids();
            let current_timestamp = current_timestamp_sec();
            for account_id in account_ids.into_iter() {
                if group.remove_member(&account_id) {
                    self.remove_user_role_group(&account_id, id);
                    self.remove_wallet_reward(&account_id, rewards.as_slice(), current_timestamp);
                };
            }
            self.groups.insert(&id, &group);
            true
        } else {
            false
        }
    }
    /// Remove provided roles from the group and from the members with the role.
    /// Include removal of rewards for the removed roles.
    /// Role id with value == 0 is ignored.
    pub fn group_remove_roles(&mut self, id: GroupId, roles: Vec<RoleId>) -> bool {
        if let Some(mut group) = self.groups.get(&id) {
            let mut group_roles = self
                .group_roles
                .get(&id)
                .expect("internal - group roles not found");
            let rewards: Vec<(u16, u16)> = group.group_reward_ids();
            let current_timestamp = current_timestamp_sec();
            let mut user_cache: HashMap<AccountId, (UserRoles, Vec<(u16, u16)>)> = HashMap::new();
            for role_id in roles {
                if role_id == 0 {
                    continue;
                }
                if group_roles.remove(role_id) {
                    let rewards_to_remove: Vec<(u16, u16)> = rewards
                        .clone()
                        .into_iter()
                        .filter(|(_, r)| *r == role_id)
                        .collect();
                    for (reward_id, _) in rewards_to_remove.iter() {
                        group.remove_group_reward_id(*reward_id);
                    }
                    let members_with_the_role =
                        self.get_group_members_with_role(id, &group, role_id);
                    for account_id in members_with_the_role {
                        if let Some((user_role, rewards)) = user_cache.get_mut(&account_id) {
                            user_role.remove_group_role(id, role_id);
                            rewards.append(&mut rewards_to_remove.clone());
                        } else {
                            let mut user_role = self
                                .user_roles
                                .get(&account_id)
                                .expect("internal - user roles not found");
                            user_role.remove_group_role(id, role_id);
                            user_cache.insert(account_id, (user_role, rewards_to_remove.clone()));
                        }
                    }
                }
            }
            for (account_id, (user_roles, rewards_to_remove)) in user_cache.into_iter() {
                self.save_user_roles(&account_id, &user_roles);
                self.remove_wallet_reward(
                    &account_id,
                    rewards_to_remove.as_slice(),
                    current_timestamp,
                );
            }
            self.groups.insert(&id, &group);
            self.group_roles.insert(&id, &group_roles);
            true
        } else {
            false
        }
    }
    /// Remove roles for provided members.
    /// Include removal of rewards for the removed roles.
    pub fn group_remove_member_roles(
        &mut self,
        id: GroupId,
        member_roles: Vec<MemberRoles>,
    ) -> bool {
        if let Some(group) = self.groups.get(&id) {
            let group_roles = self
                .group_roles
                .get(&id)
                .expect("internal - group roles not found");
            let rewards: Vec<(u16, u16)> = group.group_reward_ids();
            let current_timestamp = current_timestamp_sec();
            let mut user_cache: HashMap<AccountId, (UserRoles, Vec<(u16, u16)>)> = HashMap::new();
            for role in member_roles {
                if let Some(role_id) = group_roles.find_role_by_name(role.name.as_str()) {
                    let rewards_to_remove: Vec<(u16, u16)> = rewards
                        .clone()
                        .into_iter()
                        .filter(|(_, r)| *r == role_id)
                        .collect();
                    for account_id in role.members {
                        if let Some((user_role, rewards)) = user_cache.get_mut(&account_id) {
                            user_role.remove_group_role(id, role_id);
                            rewards.append(&mut rewards_to_remove.clone());
                        } else {
                            let mut user_role = self
                                .user_roles
                                .get(&account_id)
                                .expect("internal - user roles not found");
                            user_role.remove_group_role(id, role_id);
                            user_cache.insert(account_id, (user_role, rewards_to_remove.clone()));
                        }
                    }
                }
            }
            for (account_id, (user_roles, rewards_to_remove)) in user_cache.into_iter() {
                self.save_user_roles(&account_id, &user_roles);
                self.remove_wallet_reward(
                    &account_id,
                    rewards_to_remove.as_slice(),
                    current_timestamp,
                );
            }
            true
        } else {
            false
        }
    }

    pub fn group_remove(&mut self, id: GroupId) {
        if let Some(group) = self.groups.get(&id) {
            let rewards: Vec<(u16, u16)> = group.group_reward_ids();
            let current_timestamp = current_timestamp_sec();
            for account_id in group.get_members_accounts_refs() {
                self.remove_user_role_group(&account_id, id);
                self.remove_wallet_reward(account_id, rewards.as_slice(), current_timestamp);
            }
            self.group_roles.remove(&id);
            self.groups.remove(&id);
        }
    }
    /// TODO: Refactor.
    /// Add group roles and user roles to self.
    /// Return Err if user roles do not match.
    pub fn internal_add_group_roles(
        &mut self,
        group_id: u16,
        group_members: &[GroupMember],
        group_member_roles: Vec<MemberRoles>,
    ) {
        let mut cache = HashMap::with_capacity(group_members.len());
        for member in group_members.iter() {
            let user_roles = self
                .user_roles
                .get(&member.account_id)
                .unwrap_or_else(UserRoles::default);
            cache.insert(&member.account_id, user_roles);
        }
        let mut group_roles = Roles::new();
        for member_role in group_member_roles.into_iter() {
            if let Some(role_id) = group_roles.insert(member_role.name) {
                for member in member_role.members {
                    let user_roles = cache.get_mut(&member).expect("User roles do not match.");
                    user_roles.add_group_role(self.group_last_id, role_id);
                }
            }
        }
        for (account, mut roles) in cache.into_iter() {
            roles.add_group_role(group_id, 0);
            self.save_user_roles(account, &roles);
        }
        self.group_roles.insert(&self.group_last_id, &group_roles);
    }
}
