use std::mem::take;

use library::{types::DataType, ObjectValues};

use crate::error::ActionError;

/// Helper method for fetching `DataType` from the object on index idx.
/// Replaces the value with `DataType`s default value.
pub fn get_datatype(object: &mut Vec<DataType>, idx: usize) -> Result<DataType, ActionError> {
    Ok(take(object.get_mut(idx).ok_or(ActionError::Binding)?))
}

pub fn get_datatype_from_values(
    object: &mut ObjectValues,
    obj_idx: usize,
    idx: usize,
) -> Result<DataType, ActionError> {
    Ok(take(
        object
            .get_mut(obj_idx)
            .ok_or(ActionError::Binding)?
            .get_mut(idx)
            .ok_or(ActionError::Binding)?,
    ))
}

/// Deserialize helpers for DAO's structs.
pub mod deserialize {
    use std::convert::TryFrom;

    use super::{get_datatype, get_datatype_from_values};
    use library::types::DataType;
    use near_sdk::AccountId;

    use crate::{
        error::ActionError,
        group::{GroupInput, GroupMember, GroupSettings, GroupTokenLockInput},
        settings::DaoSettings,
        token_lock::{UnlockMethod, UnlockPeriodInput},
    };

    pub fn deserialize_group_settings(
        user_inputs: &mut Vec<Vec<DataType>>,
        obj_idx: usize,
    ) -> Result<GroupSettings, ActionError> {
        let first_object = user_inputs.get_mut(obj_idx).ok_or(ActionError::Binding)?;

        let leader = match get_datatype(first_object, 1)? {
            DataType::Null => None,
            DataType::String(s) => {
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
        user_inputs: &mut Vec<Vec<DataType>>,
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
        user_inputs: &mut Vec<Vec<DataType>>,
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

        let token_lock = if get_datatype(token_lock_obj, 0)? == DataType::Null {
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
        user_inputs: &mut Vec<Vec<DataType>>,
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
}
