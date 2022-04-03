use std::collections::HashMap;

use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::LazyOption,
    env,
    serde::{Deserialize, Serialize},
    AccountId, IntoStorageKey,
};

use crate::token_lock::{TokenLock, UnlockMethod, UnlockPeriodInput};
use crate::{error::ERR_INVALID_AMOUNT, token_lock::UnlockPeriod};
use crate::{GroupId, TagId};

#[derive(Serialize, Deserialize)]
#[cfg_attr(
    not(target_arch = "wasm32"),
    derive(Debug, PartialEq, Clone, PartialOrd, Ord, Eq)
)]
#[serde(crate = "near_sdk::serde")]
pub struct GroupMember {
    pub account_id: String,
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
        match self.0.insert(member.account_id, member.tags) {
            Some(_) => true,
            None => false,
        }
    }

    pub fn remove_member(&mut self, account_id: AccountId) -> Option<GroupMember> {
        match self.0.remove(&account_id) {
            Some(tags) => Some(GroupMember { account_id, tags }),
            _ => None,
        }
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
    pub leader: String,
}

#[derive(Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq, Serialize))]
#[serde(crate = "near_sdk::serde")]
pub struct GroupTokenLockInput {
    pub amount: u32,
    pub start_from: u64,
    pub duration: u64,
    pub init_distribution: u32,
    pub unlock_interval: u32,
    pub periods: Vec<UnlockPeriodInput>,
}

#[derive(Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq, Serialize))]
#[serde(crate = "near_sdk::serde")]
pub struct GroupInput {
    pub settings: GroupSettings,
    pub members: Vec<GroupMember>,
    pub token_lock: GroupTokenLockInput,
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct GroupOutput {
    pub id: GroupId,
    pub settings: GroupSettings,
    pub members: Vec<GroupMember>,
    pub token_lock: TokenLock,
}

impl GroupOutput {
    pub fn from_group(id: GroupId, group: Group) -> Self {
        let token_lock = group.token_lock.get().unwrap();

        GroupOutput {
            id,
            settings: group.settings,
            members: group.members.get_members(),
            token_lock,
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Group {
    pub settings: GroupSettings,
    pub members: GroupMembers,
    pub token_lock: LazyOption<TokenLock>,
}

impl Group {
    pub fn new<T: IntoStorageKey>(
        release_key: T,
        settings: GroupSettings,
        members: Vec<GroupMember>,
        token_lock: TokenLock,
    ) -> Self {
        assert!(
            members.iter().any(|m| settings.leader == m.account_id),
            "Leader must be contained in group members"
        );
        Group {
            settings,
            members: members.into(),
            token_lock: LazyOption::new(release_key.into_storage_key(), Some(&token_lock)),
        }
    }

    /// Adds members to the group.
    /// Returns count of new members added.
    pub fn add_members(&mut self, members: Vec<GroupMember>) -> u32 {
        let mut new_added = 0;
        for m in members.into_iter() {
            if !self.members.add_member(m) {
                new_added += 1;
            }
        }
        new_added
    }

    pub fn remove_member(&mut self, account_id: AccountId) -> Option<GroupMember> {
        self.members.remove_member(account_id)
    }

    //TODO test if storage removed properly
    pub fn remove_storage_data(&mut self) -> TokenLock {
        let token_lock = self.token_lock.get().unwrap();
        self.token_lock.remove();
        token_lock
    }

    // TODO: fix
    pub fn unlock_ft(&mut self, current_time: u64) -> u32 {
        todo!()
        /*         let mut release = self.release.get().unwrap();
        let (model, mut db): (ReleaseModel, ReleaseDb) =
            (release.model.into(), release.data.into());

        if db.total == db.unlocked {
            return 0;
        }

        //TODO
        let total_released_now = model.release(
            db.total,
            db.init_distribution,
            db.unlocked,
            (current_time / 10u64.pow(9)) as u32,
        );

        let unlocked = if total_released_now > 0 {
            let delta = total_released_now - (db.unlocked - db.init_distribution);
            db.unlocked += delta;
            delta
        } else {
            total_released_now
        };

        release.model = VReleaseModel::Curr(model);
        release.data = VReleaseDb::Curr(db);

        self.release.set(&release);
        unlocked */
    }

    pub fn get_members_accounts(&self) -> Vec<AccountId> {
        self.members
            .get_members()
            .into_iter()
            .map(|member| member.account_id)
            .collect()
    }

    pub fn get_member_by_account(&self, account_id: &str) -> Option<GroupMember> {
        self.members
            .get_members()
            .into_iter()
            .find(|m| m.account_id == account_id)
    }

    pub fn get_members_accounts_by_role(&self, role: TagId) -> Vec<AccountId> {
        self.members
            .get_members()
            .into_iter()
            .filter(|m| m.tags.iter().any(|r| *r == role))
            .map(|m| m.account_id)
            .collect()
    }

    pub fn distribute_ft(&mut self, amount: u32) -> bool {
        let mut token_lock = self.token_lock.get().unwrap();

        if token_lock.distribute(amount) {
            self.token_lock.set(&token_lock);
            true
        } else {
            false
        }
    }
}
