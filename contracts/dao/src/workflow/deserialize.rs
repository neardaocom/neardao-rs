//! Deserialize DAO internal objects from user input.

use std::convert::TryFrom;

use library::{
    locking::{LockInput, UnlockMethod, UnlockPeriodInput, UnlockingInput},
    types::activity_input::ActivityInput,
};
use near_sdk::{env, AccountId};

use crate::{
    group::{GroupInput, GroupMember, GroupSettings},
    media::{CIDInfo, Media, ResourceType},
    reward::{Reward, RewardType, RewardUserActivity, RewardWage},
    role::MemberRoles,
    treasury::{Asset, PartitionAssetInput, TreasuryPartition, TreasuryPartitionInput},
    RoleId,
};

use super::error::DeserializeError;

pub fn deser_asset(
    prefix: &str,
    action_input: &mut dyn ActivityInput,
) -> Result<Option<Asset>, DeserializeError> {
    let mut key = String::with_capacity(prefix.len() + 16);
    key.push_str(prefix);
    key.push_str(".near");
    if let Some(_) = action_input.take(&key) {
        let asset = Asset::new_near();
        Ok(Some(asset))
    } else {
        key.clear();
        key.push_str(prefix);
        key.push_str(".ft.account_id");
        if let Some(v) = action_input.take(&key) {
            let account_string = v.to_owned().try_into_string()?;
            let account_id = AccountId::try_from(account_string)?;
            key.clear();
            key.push_str(prefix);
            key.push_str(".ft.decimals");
            let decimals = action_input
                .take(&key)
                .ok_or(DeserializeError::MissingInputKey("decimals".into()))?
                .try_into_u64()? as u8;
            let asset = Asset::new_ft(account_id, decimals);
            Ok(Some(asset))
        } else {
            Ok(None)
        }
    }
}

pub fn deser_partition(
    action_input: &mut dyn ActivityInput,
) -> Result<TreasuryPartition, DeserializeError> {
    let name = action_input
        .take(&"name")
        .ok_or(DeserializeError::MissingInputKey("name".into()))?
        .try_into_string()?;
    let assets = deser_partition_assets("assets", action_input)?;
    let partition = TreasuryPartition::try_from(TreasuryPartitionInput { name, assets })
        .map_err(|_| DeserializeError::Conversion("treasury partition".into()))?;
    Ok(partition)
}

pub fn deser_partition_assets(
    prefix: &str,
    action_input: &mut dyn ActivityInput,
) -> Result<Vec<PartitionAssetInput>, DeserializeError> {
    let mut assets = vec![];
    let mut i = 0;
    loop {
        let key_asset = format!("{}.{}.asset_id", prefix, i);
        if let Some(asset_id) = deser_asset(key_asset.as_str(), action_input)? {
            let key_init_amount = format!("{}.{}.unlocking.amount_init_unlock", prefix, i);
            let amount_init_unlock = action_input
                .take(&key_init_amount)
                .ok_or(DeserializeError::MissingInputKey(
                    "amount_init_unlock".into(),
                ))?
                .try_into_u64()? as u32;
            let key_lock = format!("{}.{}.unlocking.lock", prefix, i);
            let lock_input = deser_lock_input(&key_lock, action_input)?;
            let unlocking = UnlockingInput {
                amount_init_unlock,
                lock: lock_input,
            };
            assets.push(PartitionAssetInput {
                asset_id,
                unlocking,
            });
            i += 1;
        } else {
            break;
        }
    }
    Ok(assets)
}

pub fn deser_reward(action_input: &mut dyn ActivityInput) -> Result<Reward, DeserializeError> {
    let name = action_input
        .take(&"name")
        .ok_or(DeserializeError::MissingInputKey("name".into()))?
        .try_into_string()?;
    let group_id = action_input
        .take(&"group_id")
        .ok_or(DeserializeError::MissingInputKey("group_id".into()))?
        .try_into_u64()? as u16;
    let role_id = action_input
        .take(&"role_id")
        .ok_or(DeserializeError::MissingInputKey("role_id".into()))?
        .try_into_u64()? as u16;
    let partition_id = action_input
        .take(&"partition_id")
        .ok_or(DeserializeError::MissingInputKey("partition_id".into()))?
        .try_into_u64()? as u16;
    let reward_object = if let Some(v) = action_input.take(&"type.wage.unit_seconds") {
        let unit_seconds = v.try_into_u64()? as u16;
        RewardType::Wage(RewardWage { unit_seconds })
    } else if let Some(v) = action_input.take(&"type.user_activity.activity_ids") {
        let activity_ids = v
            .to_owned()
            .try_into_vec_u64()?
            .into_iter()
            .map(|e| e as u8)
            .collect();
        RewardType::UserActivity(RewardUserActivity { activity_ids })
    } else {
        env::panic_str("try_bind_reward - invalid reward type");
    };
    let time_valid_from = action_input
        .take(&"time_valid_from")
        .ok_or(DeserializeError::MissingInputKey("time_valid_from".into()))?
        .try_into_u64()? as u64;
    let time_valid_to = action_input
        .take(&"time_valid_to")
        .ok_or(DeserializeError::MissingInputKey("time_valid_to".into()))?
        .try_into_u64()? as u64;
    let reward_amounts = deser_reward_amounts("reward_amounts", action_input)?;
    let reward = Reward::new(
        name,
        group_id,
        role_id,
        partition_id,
        reward_object,
        reward_amounts,
        time_valid_from,
        time_valid_to,
    );
    Ok(reward)
}
fn deser_reward_amounts(
    prefix: &str,
    action_input: &mut dyn ActivityInput,
) -> Result<Vec<(Asset, u128)>, DeserializeError> {
    let mut reward_assets = vec![];
    let mut i = 0;
    loop {
        let key_asset = format!("{}.{}.0", prefix, i);
        if let Some(asset) = deser_asset(key_asset.as_str(), action_input)? {
            let key_amount = format!("{}.{}.1", prefix, i);
            let amount = action_input
                .take(&key_amount)
                .ok_or(DeserializeError::MissingInputKey(
                    "reward_amounts.amount".into(),
                ))?
                .try_into_u128()?;
            reward_assets.push((asset, amount));
            i += 1;
        } else {
            break;
        }
    }
    Ok(reward_assets)
}

fn deser_lock_input(
    prefix: &str,
    action_input: &mut dyn ActivityInput,
) -> Result<Option<LockInput>, DeserializeError> {
    let key_amount_total_lock = format!("{}.amount_total_lock", prefix);
    if let Some(v) = action_input.take(&key_amount_total_lock) {
        let amount_total_lock = v.try_into_u128()? as u32;
        let key_start_from = format!("{}.start_from", prefix);
        let start_from = action_input
            .take(&key_start_from)
            .ok_or(DeserializeError::MissingInputKey("start_from".into()))?
            .try_into_u128()? as u64;
        let key_duration = format!("{}.duration", prefix);
        let duration = action_input
            .take(&key_duration)
            .ok_or(DeserializeError::MissingInputKey("duration".into()))?
            .try_into_u128()? as u64;
        let prefix = format!("{}.periods", prefix);
        let periods = deser_periods(prefix.as_str(), action_input)?;
        Ok(Some(LockInput {
            amount_total_lock,
            start_from,
            duration,
            periods,
        }))
    } else {
        Ok(None)
    }
}
fn deser_periods(
    prefix: &str,
    action_input: &mut dyn ActivityInput,
) -> Result<Vec<UnlockPeriodInput>, DeserializeError> {
    let mut periods = vec![];
    let mut i = 0;
    loop {
        let key_type = format!("{}.{}.type", prefix, i);
        if let Some(v) = action_input.take(&key_type) {
            let r#type = UnlockMethod::from(v.try_into_str()?);
            let key_duration = format!("{}.{}.duration", prefix, i);
            let duration = action_input
                .take(&key_duration)
                .ok_or(DeserializeError::MissingInputKey("duration".into()))?
                .try_into_u64()?;
            let key_amount = format!("{}.{}.amount", prefix, i);
            let amount = action_input
                .take(&key_amount)
                .ok_or(DeserializeError::MissingInputKey("amount".into()))?
                .try_into_u64()? as u32;
            periods.push(UnlockPeriodInput {
                r#type,
                duration,
                amount,
            });
            i += 1;
        } else {
            break;
        }
    }
    Ok(periods)
}

pub fn deser_group_input(
    action_input: &mut dyn ActivityInput,
) -> Result<GroupInput, DeserializeError> {
    let settings = deser_group_settins("settings", action_input)?;
    let members = deser_group_members("members", action_input)?;
    let member_roles = deser_member_roles("member_roles", action_input)?;

    Ok(GroupInput {
        settings,
        members,
        member_roles,
    })
}

fn deser_group_settins(
    prefix: &str,
    action_input: &mut dyn ActivityInput,
) -> Result<GroupSettings, DeserializeError> {
    let key_name = format!("{}.name", prefix);
    let name = action_input
        .take(&key_name)
        .ok_or(DeserializeError::MissingInputKey("duration".into()))?
        .try_into_string()?;
    let key_leader = format!("{}.leader", prefix);
    let leader = if let Some(v) = action_input.take(&key_leader) {
        let leader_string = v.try_into_string()?;
        Some(AccountId::try_from(leader_string)?)
    } else {
        None
    };
    let key_parent_group = format!("{}.parent_group", prefix);
    let parent_group = if let Some(v) = action_input.take(&key_parent_group) {
        v.try_into_u64()? as u16
    } else {
        0
    };

    Ok(GroupSettings {
        name,
        leader,
        parent_group,
    })
}
pub fn deser_group_members(
    prefix: &str,
    action_input: &mut dyn ActivityInput,
) -> Result<Vec<GroupMember>, DeserializeError> {
    let mut members = vec![];
    let mut i = 0;
    loop {
        let key_account_id = format!("{}.{}.account_id", prefix, i);
        if let Some(v) = action_input.take(&key_account_id) {
            let member_string = v.try_into_string()?;
            let account_id = AccountId::try_from(member_string)?;
            let key_tags = format!("{}.{}.tags", prefix, i);
            let tags = if let Some(v) = action_input.take(&key_tags) {
                v.try_into_vec_u64()?
                    .into_iter()
                    .map(|t| t as u16)
                    .collect()
            } else {
                vec![]
            };

            members.push(GroupMember { account_id, tags });
            i += 1;
        } else {
            break;
        }
    }
    Ok(members)
}
pub fn deser_member_roles(
    prefix: &str,
    action_input: &mut dyn ActivityInput,
) -> Result<Vec<MemberRoles>, DeserializeError> {
    let mut member_roles = vec![];
    let mut i = 0;
    loop {
        let key_name = format!("{}.{}.name", prefix, i);
        if let Some(v) = action_input.take(&key_name) {
            let name = v.try_into_string()?;

            let mut members = vec![];
            let mut j = 0;
            loop {
                let key_account = format!("{}.{}.members.{}", prefix, i, j);
                if let Some(v) = action_input.take(&key_account) {
                    let member_string = v.try_into_string()?;
                    let account_id = AccountId::try_from(member_string)?;
                    members.push(account_id);
                    j += 1;
                } else {
                    break;
                }
            }
            member_roles.push(MemberRoles { name, members });
            i += 1;
        } else {
            break;
        }
    }
    Ok(member_roles)
}

pub fn deser_account_ids(
    prefix: &str,
    action_input: &mut dyn ActivityInput,
) -> Result<Vec<AccountId>, DeserializeError> {
    let mut accounts = vec![];
    let mut i = 0;
    loop {
        let key_name = format!("{}.{}", prefix, i);
        if let Some(v) = action_input.take(&key_name) {
            let account_id_string = v.try_into_string()?;
            let account_id = AccountId::try_from(account_id_string)?;
            accounts.push(account_id);
            i += 1;
        } else {
            break;
        }
    }
    Ok(accounts)
}

pub fn deser_roles_ids(
    prefix: &str,
    action_input: &mut dyn ActivityInput,
) -> Result<Vec<RoleId>, DeserializeError> {
    if let Some(v) = action_input.take(&prefix) {
        let roles = v
            .try_into_vec_u64()?
            .into_iter()
            .map(|r| r as u16)
            .collect();
        Ok(roles)
    } else {
        Ok(vec![])
    }
}

pub fn deser_u64(
    prefix: &str,
    action_input: &mut dyn ActivityInput,
) -> Result<u64, DeserializeError> {
    let id = action_input
        .get(prefix)
        .ok_or(DeserializeError::MissingInputKey(prefix.into()))?
        .try_into_u64()?;
    Ok(id)
}

pub fn deser_u128(
    prefix: &str,
    action_input: &mut dyn ActivityInput,
) -> Result<u128, DeserializeError> {
    let u128 = action_input
        .get(prefix)
        .ok_or(DeserializeError::MissingInputKey(prefix.into()))?
        .try_into_u128()?;
    Ok(u128)
}

pub fn deser_media(
    prefix: &str,
    action_input: &mut dyn ActivityInput,
) -> Result<Media, DeserializeError> {
    let mut key = String::with_capacity(prefix.len() + 16);
    key.push_str(prefix);
    if prefix.len() > 0 {
        key.push('.');
    }
    key.push_str("name");
    let name = action_input
        .take(&key)
        .ok_or(DeserializeError::MissingInputKey(key.to_string()))?
        .try_into_string()?;
    key.clear();
    key.push_str(prefix);
    if prefix.len() > 0 {
        key.push('.');
    }
    key.push_str("category");
    let category = action_input
        .take(&key)
        .ok_or(DeserializeError::MissingInputKey(key.to_string()))?
        .try_into_string()?;
    key.clear();
    key.push_str(prefix);
    if prefix.len() > 0 {
        key.push('.');
    }
    key.push_str("type");
    let r#type = deser_media_type(key.as_str(), action_input)?;
    key.clear();
    key.push_str(prefix);
    if prefix.len() > 0 {
        key.push('.');
    }
    key.push_str("tags");
    let tags = action_input
        .take(&key)
        .ok_or(DeserializeError::MissingInputKey(key.to_string()))?
        .try_into_vec_u64()?
        .into_iter()
        .map(|e| e as u16)
        .collect();
    key.clear();
    key.push_str(prefix);
    if prefix.len() > 0 {
        key.push('.');
    }
    key.push_str("version");
    let version = action_input
        .take(&key)
        .ok_or(DeserializeError::MissingInputKey(key.to_string()))?
        .try_into_string()?;
    key.clear();
    key.push_str(prefix);
    if prefix.len() > 0 {
        key.push('.');
    }
    key.push_str("valid");
    let valid = action_input
        .take(&key)
        .ok_or(DeserializeError::MissingInputKey(key.to_string()))?
        .try_into_bool()?;
    Ok(Media {
        proposal_id: None,
        name,
        category,
        r#type,
        tags,
        version,
        valid,
    })
}

pub fn deser_media_type(
    prefix: &str,
    action_input: &mut dyn ActivityInput,
) -> Result<ResourceType, DeserializeError> {
    let mut key = String::with_capacity(prefix.len() + 24);
    key.push_str(prefix);
    key.push_str(".text");
    if let Some(v) = action_input.take(&key) {
        let text = v.try_into_string()?;
        let resource_type = ResourceType::Text(text);
        return Ok(resource_type);
    }
    key.clear();
    key.push_str(prefix);
    key.push_str(".link");
    if let Some(v) = action_input.take(&key) {
        let link = v.try_into_string()?;
        let resource_type = ResourceType::Link(link);
        return Ok(resource_type);
    }
    key.clear();
    key.push_str(prefix);
    key.push_str(".cid.ipfs");
    if let Some(v) = action_input.take(&key) {
        let ipfs = v.try_into_string()?;
        key.clear();
        key.push_str(prefix);
        key.push_str(".cid.cid");
        let cid = action_input
            .take(&key)
            .ok_or(DeserializeError::MissingInputKey("cid.cid".into()))?
            .try_into_string()?;
        key.clear();
        key.push_str(prefix);
        key.push_str(".cid.mimetype");
        let mimetype = action_input
            .take(&key)
            .ok_or(DeserializeError::MissingInputKey("cid.mimetype".into()))?
            .try_into_string()?;
        let resource_type = ResourceType::Cid(CIDInfo {
            ipfs,
            cid,
            mimetype,
        });
        return Ok(resource_type);
    }
    return Err(DeserializeError::MissingInputKey("media type".into()));
}
