use library::workflow::types::ActivityRight;
use near_sdk::AccountId;

use crate::core::Contract;

pub mod utils {
    use library::functions::utils::{
        into_storage_key_wrapper_str, into_storage_key_wrapper_u16, StorageKeyWrapper,
    };
    use near_sdk::env;

    use crate::{
        constants::{GROUP_RELEASE_PREFIX, STORAGE_BUCKET_PREFIX},
        GroupId, TimestampSec,
    };

    pub fn get_group_key(id: GroupId) -> StorageKeyWrapper {
        into_storage_key_wrapper_u16(GROUP_RELEASE_PREFIX, id)
    }

    pub fn get_bucket_id(id: &str) -> StorageKeyWrapper {
        into_storage_key_wrapper_str(STORAGE_BUCKET_PREFIX, id)
    }

    pub fn current_timestamp_sec() -> TimestampSec {
        env::block_timestamp() / 10u64.pow(9)
    }
}

impl Contract {
    pub fn check_rights(&self, rights: &[ActivityRight], account_id: &AccountId) -> bool {
        if rights.is_empty() {
            return true;
        }
        for right in rights.iter() {
            match right {
                ActivityRight::Anyone => {
                    return true;
                }
                ActivityRight::Group(g) => match self.groups.get(g) {
                    Some(group) => {
                        if group.is_member(account_id) {
                            return true;
                        } else {
                            continue;
                        }
                    }
                    _ => continue,
                },
                ActivityRight::GroupMember(g, name) => {
                    if *name != *account_id {
                        continue;
                    }
                    match self.groups.get(g) {
                        Some(group) => match group.is_member(account_id) {
                            true => return true,
                            false => continue,
                        },
                        _ => continue,
                    }
                }
                ActivityRight::TokenHolder => {
                    if self.delegations.get(account_id).unwrap_or(0) > 0 {
                        return true;
                    } else {
                        continue;
                    }
                }
                ActivityRight::GroupRole(g, r) => match self.user_roles.get(account_id) {
                    Some(roles) => {
                        if roles.has_group_role(*g, *r) {
                            return true;
                        } else {
                            continue;
                        }
                    }
                    _ => continue,
                },
                ActivityRight::GroupLeader(g) => match self.groups.get(g) {
                    Some(group) => {
                        if group.is_account_id_leader(account_id) {
                            return true;
                        } else {
                            continue;
                        }
                    }
                    _ => continue,
                },
                ActivityRight::Member => {
                    if self.user_roles.get(&account_id).is_some() {
                        return true;
                    } else {
                        continue;
                    }
                }
                ActivityRight::Account(a) => match *a == *account_id {
                    true => return true,
                    false => continue,
                },
            }
        }
        false
    }
}
