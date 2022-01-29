use std::collections::HashMap;

use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::LazyOption,
    serde::{Deserialize, Serialize},
    AccountId, IntoStorageKey, env,
};

use crate::errors::ERR_INVALID_AMOUNT;
use crate::release::{
    Release, ReleaseDb, ReleaseModel, ReleaseModelInput, VReleaseDb, VReleaseModel,
};
use crate::{GroupId, TagId};

#[derive(Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct GroupMember {
    pub account_id: String,
    pub tags: Vec<TagId>,
}

//#[repr(transparent)]
#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct GroupMembers(HashMap<AccountId, Vec<TagId>>);

impl GroupMembers {
    /// Adds members to group
    /// Overrides existing
    pub fn add_member(&mut self, member: GroupMember) {
        self.0.insert(member.account_id, member.tags);
    }

    pub fn remove_member(&mut self, account_id: AccountId) -> Option<GroupMember> {
        match self.0.remove(&account_id) {
            Some(tags) => Some(GroupMember { account_id, tags }),
            _ => None,
        }
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

#[derive(Deserialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct GroupReleaseInput {
    pub amount: u32,
    pub init_distribution: u32,
    pub start_from: u32,
    pub duration: u32,
    pub model: ReleaseModelInput,
}

impl From<GroupReleaseInput> for Release {
    fn from(input: GroupReleaseInput) -> Self {
        assert!(
            input.amount >= input.init_distribution,
            "{}",
            ERR_INVALID_AMOUNT
        );
        let rel_model = match input.model {
            ReleaseModelInput::Linear => ReleaseModel::Linear {
                duration: input.duration,
                release_end: input.start_from + input.duration,
            },
            ReleaseModelInput::None => ReleaseModel::None,
        };

        let rel_db = ReleaseDb::new(
            input.amount,
            input.init_distribution,
            input.init_distribution,
        );

        Release {
            model: VReleaseModel::Curr(rel_model),
            data: VReleaseDb::Curr(rel_db),
        }
    }
}

#[derive(Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq, Serialize))]
#[serde(crate = "near_sdk::serde")]
pub struct GroupInput {
    pub settings: GroupSettings,
    pub members: Vec<GroupMember>,
    pub release: GroupReleaseInput,
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct GroupOutput {
    pub id: GroupId,
    pub settings: GroupSettings,
    pub members: Vec<GroupMember>,
    pub release_model: ReleaseModel,
    pub release_data: ReleaseDb,
}

impl GroupOutput {
    pub fn from_group(id: GroupId, group: Group) -> Self {
        let release = group.release.get().unwrap();

        GroupOutput {
            id,
            settings: group.settings,
            members: group.members.get_members(),
            release_model: release.model.into(),
            release_data: release.data.into(),
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Group {
    pub key: Vec<u8>,
    pub settings: GroupSettings,
    pub members: GroupMembers,
    pub release: LazyOption<Release>,
}

impl Group {
    pub fn new<T: IntoStorageKey>(
        release_key: T,
        settings: GroupSettings,
        members: Vec<GroupMember>,
        release: Release,
    ) -> Self {
        let key = release_key.into_storage_key();
        Group {
            key: key.clone(),
            settings,
            members: members.into(),
            release: LazyOption::new(key, Some(&release)),
        }
    }

    pub fn add_members(&mut self, members: Vec<GroupMember>) {
        for m in members.into_iter() {
            self.members.add_member(m)
        }
    }

    pub fn remove_member(&mut self, account_id: AccountId) -> Option<GroupMember> {
        self.members.remove_member(account_id)
    }

    pub fn remove_storage_data(&mut self) -> Release {
        let release = self.release.get().unwrap();
        self.release.remove();
        env::storage_remove(&self.key);
        release
    }

    pub fn unlock_ft(&mut self, current_time: u64) -> u32 {
        let mut release = self.release.get().unwrap();
        let (model, mut db): (ReleaseModel, ReleaseDb) = (release.model.into(), release.data.into());


        if db.total == db.unlocked {
            return 0;
        }

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
        unlocked
    }

    pub fn distribute_ft(&mut self, amount: u32) -> bool {
        let mut release = self.release.get().unwrap();
        let mut db: ReleaseDb = release.data.into();

        match db.distributed + amount <= db.unlocked {
            true => {
                db.distributed += amount;
                release.data = VReleaseDb::Curr(db);
                self.release.set(&release);
                true
            },
            _ => false,
        }
    }
}
