// TODO: Finish implementations.
// TODO: Integration tests.
/// Deserialize helpers for DAO's structs.
pub mod deserialize {
    use std::convert::TryFrom;

    use library::types::activity_input::ActivityInput;
    use near_sdk::{env, AccountId};

    use crate::{
        internal::utils::current_timestamp_sec,
        reward::{Reward, RewardType, RewardUserActivity, RewardWage},
        treasury::{Asset, PartitionAsset, TreasuryPartition},
    };

    pub fn bind_asset(
        asset_type: &str,
        prefix: &str,
        action_input: &mut dyn ActivityInput,
    ) -> Asset {
        match asset_type {
            "ft" => {
                let ft_account_key = format!("{}.ft_account_id", prefix);
                let ft_account_string = action_input
                    .get(&ft_account_key)
                    .expect("binding - missing key: asset_ft")
                    .to_owned()
                    .try_into_string()
                    .expect("invalid datatype: asset_ft");
                let ft_account = AccountId::try_from(ft_account_string)
                    .expect("binding - ft: failed to parse account id");
                let decimals_key = format!("{}.decimals", prefix);
                let decimals = action_input
                    .get(&decimals_key)
                    .expect("binding - missing key: asset_ft_decimals")
                    .try_into_u64()
                    .expect("invalid datatype: asset_decimals")
                    as u8;
                Asset::new_ft(ft_account, decimals)
            }
            "nft" => {
                todo!()
            }
            "near" => Asset::Near,
            _ => env::panic_str("unsupported asset type"),
        }
    }

    // TODO: Checks for unique assets.
    pub fn try_bind_partition(action_input: &mut dyn ActivityInput) -> Option<TreasuryPartition> {
        let asset_count = action_input
            .get(&"asset_count")
            .expect("binding - missing key: asset_count")
            .try_into_u64()
            .expect("invalid datatype: asset_count");

        // TODO: Refactor to match flattened structure.
        let mut assets = vec![];
        let current_timestamp = current_timestamp_sec();
        for i in 0..asset_count {
            let asset_key = format!("assets.{}.type", i);
            let asset_type = action_input
                .get(&asset_key)
                .expect("binding - missing key: assets.x.type")
                .to_owned()
                .try_into_string()
                .expect("invalid datatype: assets.x.type");
            let asset_amount_key = format!("assets.{}.amount", i);
            let amount = action_input
                .get(&asset_amount_key)
                .expect("binding - missing key: assets.x.amount")
                .try_into_u128()
                .expect("invalid datatype: assets.x.amount");
            let lock = None;
            let prefix = format!("assets.{}", i);
            let asset_id = bind_asset(asset_type.as_str(), prefix.as_str(), action_input);
            assets.push(PartitionAsset::new(
                asset_id,
                amount,
                lock,
                current_timestamp,
            ));
        }

        let partition = TreasuryPartition { assets };
        Some(partition)
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
        } else if let Some(v) = action_input.get(&"type.wage.user_activity") {
            let activity_ids = v
                .to_owned()
                .try_into_vec_u64()
                .expect("invalid datatype: type.name.activity_ids")
                .into_iter()
                .map(|e| e as u8)
                .collect();
            RewardType::UserActivity(RewardUserActivity { activity_ids })
        } else {
            env::panic_str("invalid reward type");
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

        let reward_count = action_input
            .get(&"reward_count")
            .expect("binding - missing key: reward_count")
            .try_into_u64()
            .expect("invalid datatype: reward_count") as usize;

        // TODO: Refactor to match flattened structure.
        let mut reward_amounts = vec![];
        for i in 0..reward_count {
            let asset_key = format!("reward_amounts.{}.type", i);
            let asset_type = action_input
                .get(&asset_key)
                .expect("binding - missing key: reward_amounts.x.type")
                .to_owned()
                .try_into_string()
                .expect("invalid datatype: reward_amounts.x.type");
            let reward_amount_key = format!("reward_amounts.{}.amount", i);
            let reward_amount = action_input
                .get(&reward_amount_key)
                .expect("binding - missing key: reward_amounts.x.amount")
                .try_into_u128()
                .expect("invalid datatype: reward_amounts.x.amount");
            let prefix = format!("reward_amounts.{}", i);
            let asset_id = bind_asset(asset_type.as_str(), prefix.as_str(), action_input);
            reward_amounts.push((asset_id, reward_amount));
        }
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
