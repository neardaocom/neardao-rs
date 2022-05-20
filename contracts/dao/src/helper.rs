// TODO: Finish implementations.
// TODO: Integration tests.
/// Deserialize helpers for DAO's structs.
pub mod deserialize {
    use std::{collections::HashMap, convert::TryFrom};

    use library::{
        locking::{LockInput, UnlockMethod, UnlockPeriodInput, UnlockingInput},
        types::activity_input::ActivityInput,
    };
    use near_sdk::{env, serde_json, AccountId};

    use crate::{
        group::{GroupInput, GroupMember, GroupSettings},
        reward::{Reward, RewardType, RewardUserActivity, RewardWage},
        role::MemberRoles,
        treasury::{Asset, PartitionAssetInput, TreasuryPartition, TreasuryPartitionInput},
        RoleId,
    };

    pub fn bind_asset(
        prefix: &str,
        action_input: &mut dyn ActivityInput,
    ) -> Result<Option<Asset>, String> {
        let mut key = String::with_capacity(prefix.len() + 16);
        key.push_str(prefix);
        key.push_str(".near");
        if let Some(_) = action_input.get(&key) {
            let asset = Asset::new_near();
            Ok(Some(asset))
        } else {
            key.clear();
            key.push_str(prefix);
            key.push_str(".ft.account_id");
            if let Some(v) = action_input.get(&key) {
                let account_string = v
                    .to_owned()
                    .try_into_string()
                    .expect("invalid datatype: asset ft account_id");
                let account_id = AccountId::try_from(account_string)
                    .expect("binding - ft: failed to parse account id");

                key.clear();
                key.push_str(prefix);
                key.push_str(".ft.decimals");
                let decimals = action_input
                    .get(&key)
                    .expect("binding - missing key: asset ft decimals")
                    .try_into_u64()
                    .expect("invalid datatype: asset ft decimals")
                    as u8;
                let asset = Asset::new_ft(account_id, decimals);
                Ok(Some(asset))
            } else {
                Ok(None)
            }
        }
    }

    // TODO: Checks for unique assets.
    pub fn try_bind_partition(action_input: &mut dyn ActivityInput) -> Option<TreasuryPartition> {
        let name = action_input
            .get(&"name")
            .expect("binding - missing key: name")
            .clone()
            .try_into_string()
            .expect("invalid datatype: name");
        let assets = bind_partition_assets("assets", action_input);
        let partition = TreasuryPartition::try_from(TreasuryPartitionInput { name, assets })
            .expect("failed to create TreasuryPartition");
        Some(partition)
    }

    pub fn bind_partition_assets(
        prefix: &str,
        action_input: &mut dyn ActivityInput,
    ) -> Vec<PartitionAssetInput> {
        let mut assets = vec![];
        let mut i = 0;
        loop {
            let key_asset = format!("{}.{}.asset_id", prefix, i);
            if let Some(asset_id) = bind_asset(key_asset.as_str(), action_input)
                .expect("failed to bind partition: asset")
            {
                let key_init_amount = format!("{}.{}.unlocking.amount_init_unlock", prefix, i);
                let amount_init_unlock = action_input
                    .get(&key_init_amount)
                    .expect("binding - missing key: unlocking.amount_init_unlock")
                    .try_into_u64()
                    .expect("invalid datatype: unlocking.amount_init_unlock")
                    as u32;
                let key_lock = format!("{}.{}.unlocking.lock", prefix, i);
                let lock_input = bind_lock_input(&key_lock, action_input);
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
        assets
    }

    // TODO: Checks.
    pub fn try_bind_reward(action_input: &mut dyn ActivityInput) -> Option<Reward> {
        let group_id = action_input
            .get(&"group_id")
            .expect("binding - missing key: group_id")
            .try_into_u64()
            .expect("invalid datatype: group_id") as u16;

        let role_id = action_input
            .get(&"role_id")
            .expect("binding - missing key: role_id")
            .try_into_u64()
            .expect("invalid datatype: role_id") as u16;

        let partition_id = action_input
            .get(&"partition_id")
            .expect("binding - missing key: partition_id")
            .try_into_u64()
            .expect("invalid datatype: partition_id") as u16;

        // Parse reward type object.
        let reward_object = if let Some(v) = action_input.get(&"type.wage.unit_seconds") {
            let unit_seconds =
                v.try_into_u64()
                    .expect("invalid datatype: type.wage.unit_seconds") as u16;
            RewardType::Wage(RewardWage { unit_seconds })
        } else if let Some(v) = action_input.get(&"type.user_activity.activity_ids") {
            let activity_ids = v
                .to_owned()
                .try_into_vec_u64()
                .expect("invalid datatype: type.user_activity.activity_ids")
                .into_iter()
                .map(|e| e as u8)
                .collect();
            RewardType::UserActivity(RewardUserActivity { activity_ids })
        } else {
            env::panic_str("try_bind_reward - invalid reward type");
        };

        let time_valid_from = action_input
            .get(&"time_valid_from")
            .expect("binding - missing key: time_valid_from")
            .try_into_u64()
            .expect("invalid datatype: time_valid_from") as u64;

        let time_valid_to = action_input
            .get(&"time_valid_to")
            .expect("binding - missing key: time_valid_to")
            .try_into_u64()
            .expect("invalid datatype: time_valid_to") as u64;

        // TODO: Refactor to match flattened structure.
        let reward_amounts = bind_reward_amounts("reward_amounts", action_input);
        let reward = Reward::new(
            group_id,
            role_id,
            partition_id,
            reward_object,
            reward_amounts,
            time_valid_from,
            time_valid_to,
        );
        Some(reward)
    }
    fn bind_reward_amounts(
        prefix: &str,
        action_input: &mut dyn ActivityInput,
    ) -> Vec<(Asset, u128)> {
        let mut reward_assets = vec![];
        let mut i = 0;
        loop {
            let key_asset = format!("{}.{}.0", prefix, i);
            if let Some(asset) = bind_asset(key_asset.as_str(), action_input)
                .expect("failed to bind reward_amounts: asset")
            {
                let key_amount = format!("{}.{}.1", prefix, i);
                let amount = action_input
                    .get(&key_amount)
                    .expect("binding - missing key: reward_amounts.amount")
                    .try_into_u128()
                    .expect("invalid datatype: reward_amounts.amount");
                reward_assets.push((asset, amount));
                i += 1;
            } else {
                break;
            }
        }
        reward_assets
    }

    fn bind_lock_input(prefix: &str, action_input: &mut dyn ActivityInput) -> Option<LockInput> {
        let key_amount_total_lock = format!("{}.amount_total_lock", prefix);
        if let Some(v) = action_input.get(&key_amount_total_lock) {
            let amount_total_lock =
                v.try_into_u128()
                    .expect("binding - missing key: amount_total_lock") as u32;
            let key_start_from = format!("{}.start_from", prefix);
            let start_from = action_input
                .get(&key_start_from)
                .expect("binding - missing key: start_from")
                .try_into_u128()
                .expect("invalid datatype: start_from") as u64;
            let key_duration = format!("{}.start_from", prefix);
            let duration = action_input
                .get(&key_duration)
                .expect("binding - missing key: duration")
                .try_into_u128()
                .expect("invalid datatype: duration") as u64;
            let prefix = format!("{}.periods", prefix);
            let periods = bind_periods(prefix.as_str(), action_input);
            Some(LockInput {
                amount_total_lock,
                start_from,
                duration,
                periods,
            })
        } else {
            None
        }
    }
    fn bind_periods(prefix: &str, action_input: &mut dyn ActivityInput) -> Vec<UnlockPeriodInput> {
        let mut periods = vec![];
        let mut i = 0;
        loop {
            let key_type = format!("{}.{}.type", prefix, i);
            if let Some(v) = action_input.get(&key_type) {
                let r#type = UnlockMethod::from(v.try_into_str().expect("invalid datatype: type"));
                let key_duration = format!("{}.{}.duration", prefix, i);
                let duration = action_input
                    .get(&key_duration)
                    .expect("binding - missing key: duration")
                    .try_into_u64()
                    .expect("invalid datatype: duration");
                let key_amount = format!("{}.{}.amount", prefix, i);
                let amount = action_input
                    .get(&key_amount)
                    .expect("binding - missing key: amount")
                    .try_into_u64()
                    .expect("invalid datatype: amount") as u32;
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
        periods
    }

    pub fn try_bind_group(action_input: &mut dyn ActivityInput) -> GroupInput {
        let settings = bind_group_settings("settings", action_input);
        let members = try_bind_group_members("members", action_input);
        let member_roles = try_to_bind_member_roles("member_roles", action_input);

        GroupInput {
            settings,
            members,
            member_roles,
        }
    }

    fn bind_group_settings(prefix: &str, action_input: &mut dyn ActivityInput) -> GroupSettings {
        let key_name = format!("{}.name", prefix);
        let name = action_input
            .get(&key_name)
            .expect("binding - missing key: name")
            .clone()
            .try_into_string()
            .expect("invalid datatype: name");
        let key_leader = format!("{}.leader", prefix);
        let leader = if let Some(v) = action_input.get(&key_leader) {
            let leader_string = v
                .clone()
                .try_into_string()
                .expect("invalid datatype: leader");
            Some(AccountId::try_from(leader_string).expect("failed to parse leader account id"))
        } else {
            None
        };
        let key_parent_group = format!("{}.parent_group", prefix);
        let parent_group = if let Some(v) = action_input.get(&key_parent_group) {
            v.try_into_u64().expect("invalid datatype: duration") as u16
        } else {
            0
        };

        GroupSettings {
            name,
            leader,
            parent_group,
        }
    }
    pub fn try_bind_group_members(
        prefix: &str,
        action_input: &mut dyn ActivityInput,
    ) -> Vec<GroupMember> {
        let mut members = vec![];
        let mut i = 0;
        loop {
            let key_account_id = format!("{}.{}.account_id", prefix, i);
            if let Some(v) = action_input.get(&key_account_id) {
                let member_string = v
                    .clone()
                    .try_into_string()
                    .expect("invalid datatype: member");
                let account_id =
                    AccountId::try_from(member_string).expect("failed to parse member account id");
                let key_tags = format!("{}.{}.tags", prefix, i);
                let tags = if let Some(v) = action_input.get(&key_tags) {
                    v.clone()
                        .try_into_vec_u64()
                        .expect("invalid datatype: tags")
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
        members
    }
    pub fn try_to_bind_member_roles(
        prefix: &str,
        action_input: &mut dyn ActivityInput,
    ) -> Vec<MemberRoles> {
        let mut member_roles = vec![];
        let mut i = 0;
        loop {
            let key_name = format!("{}.{}.name", prefix, i);
            if let Some(v) = action_input.get(&key_name) {
                let name = v
                    .clone()
                    .try_into_string()
                    .expect("invalid datatype: member roles name");

                let mut members = vec![];
                let mut j = 0;
                loop {
                    let key_account = format!("{}.{}.members.{}", prefix, i, j);
                    if let Some(v) = action_input.get(&key_account) {
                        let member_string = v
                            .clone()
                            .try_into_string()
                            .expect("invalid datatype: member name");
                        let account_id = AccountId::try_from(member_string)
                            .expect("failed to parse member account id");
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
        member_roles
    }

    pub fn try_bind_accounts(prefix: &str, action_input: &mut dyn ActivityInput) -> Vec<AccountId> {
        let mut accounts = vec![];
        let mut i = 0;
        loop {
            let key_name = format!("{}.{}", prefix, i);
            if let Some(v) = action_input.get(&key_name) {
                let account_id_string = v
                    .clone()
                    .try_into_string()
                    .expect("invalid datatype: string account id");
                let account_id =
                    AccountId::try_from(account_id_string).expect("failed to parse account id");
                accounts.push(account_id);
                i += 1;
            } else {
                break;
            }
        }
        accounts
    }

    pub fn try_to_bind_roles(prefix: &str, action_input: &mut dyn ActivityInput) -> Vec<RoleId> {
        if let Some(v) = action_input.get(&prefix) {
            let roles = v
                .clone()
                .try_into_vec_u64()
                .expect("invalid datatype: role ids vec")
                .into_iter()
                .map(|r| r as u16)
                .collect();
            roles
        } else {
            vec![]
        }
    }

    /*
    pub fn deserialize_group_settings(
        user_inputs: &mut Vec<Vec<Value>>,
        obj_idx: usize,
    ) -> Result<GroupSettings, ActionError> {
        let first_object = user_inputs.get_mut(obj_idx).ok_or(ActionError::Binding)?;

        let leader = match get_datatype(first_object, 1)? {
            Value::Null => None,
            Value::String(s) => {
                let acc = AccountId::try_from(s).map_err(|_| ActionError::InvalidDataType)?;
                Some(acc)
            }
            _ => return Err(ActionError::Binding),
        };

        // Settings
        Ok(GroupSettings {
            name: get_datatype(first_object, 0)?.try_into_string()?,
            leader,
        })
    }

    pub fn deserialize_group_members(
        user_inputs: &mut Vec<Vec<Value>>,
        obj_idx: usize,
    ) -> Result<Vec<GroupMember>, ActionError> {
        let obj = user_inputs.get_mut(obj_idx).ok_or(ActionError::Binding)?;

        if obj.len() % 2 != 0 {
            return Err(ActionError::Binding);
        }

        let mut members = Vec::with_capacity(obj.len() / 2);
        for idx in (0..obj.len()).step_by(2) {
            let tags = get_datatype(obj, idx + 1)?.try_into_vec_u64()?;

            let tags = tags.into_iter().map(|t| t as u16).collect();
            let member = get_datatype(obj, idx)?
                .try_into_string()?
                .try_into()
                .map_err(|_| ActionError::Binding)?;

            members.push(GroupMember {
                account_id: member,
                tags,
            })
        }

        Ok(members)
    }
    /// Deserializes `GroupInput` from user action inputs for GroupAdd action.
    pub fn deserialize_group_input(
        user_inputs: &mut Vec<Vec<Value>>,
    ) -> Result<GroupInput, ActionError> {
        let settings = deserialize_group_settings(user_inputs, 1)?;

        // Members - Vec<Obj>
        let members = deserialize_group_members(user_inputs, 4)?;

        // TokenLock with inner Vec<Obj>
        let unlock_period_col = user_inputs.get_mut(3).ok_or(ActionError::Binding)?;

        if unlock_period_col.len() % 3 != 0 {
            return Err(ActionError::Binding);
        }

        let mut unlock_models = Vec::with_capacity(unlock_period_col.len() / 3);
        for idx in (0..unlock_period_col.len()).step_by(3) {
            let kind =
                UnlockMethod::try_from(get_datatype(unlock_period_col, idx)?.try_into_string()?)
                    .expect("Failed to create ReleaseType from input.");

            unlock_models.push(UnlockPeriodInput {
                kind,
                duration: get_datatype(unlock_period_col, idx + 1)?.try_into_u64()?,
                amount: get_datatype(unlock_period_col, idx + 2)?.try_into_u64()? as u32,
            })
        }
        let token_lock_obj = user_inputs.get_mut(2).ok_or(ActionError::Binding)?;

        let token_lock = if get_datatype(token_lock_obj, 0)? == Value::Null {
            None
        } else {
            Some(GroupTokenLockInput {
                amount: get_datatype(token_lock_obj, 0)?.try_into_u64()? as u32,
                start_from: get_datatype(token_lock_obj, 1)?.try_into_u64()?,
                duration: get_datatype(token_lock_obj, 2)?.try_into_u64()?,
                init_distribution: get_datatype(token_lock_obj, 3)?.try_into_u64()? as u32,
                unlock_interval: get_datatype(token_lock_obj, 4)?.try_into_u64()? as u32,
                periods: unlock_models,
            })
        };

        Ok(GroupInput {
            settings,
            members,
            token_lock,
        })
    }

    pub fn deserialize_dao_settings(
        user_inputs: &mut Vec<Vec<Value>>,
    ) -> Result<DaoSettings, ActionError> {
        let tags = get_datatype_from_values(user_inputs, 0, 2)?.try_into_vec_u64()?;
        let tags = tags.into_iter().map(|t| t as u16).collect();

        let settings = DaoSettings {
            name: get_datatype_from_values(user_inputs, 0, 0)?.try_into_string()?,
            purpose: get_datatype_from_values(user_inputs, 0, 1)?.try_into_string()?,
            tags,
            dao_admin_account_id: get_datatype_from_values(user_inputs, 0, 3)?
                .try_into_string()?
                .try_into()
                .map_err(|_| ActionError::Binding)?,
            dao_admin_rights: get_datatype_from_values(user_inputs, 0, 4)?.try_into_vec_string()?,
            workflow_provider: get_datatype_from_values(user_inputs, 0, 5)?
                .try_into_string()?
                .try_into()
                .map_err(|_| ActionError::Binding)?,
        };

        Ok(settings)
    }
    */
}
