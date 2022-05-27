use std::collections::HashMap;

use library::{
    types::{activity_input::UserInput, datatype::Value},
    workflow::{
        action::{ActionInput, ActionInputType},
        types::DaoActionIdent,
    },
};

pub const GROUP1_ADD_GROUP: u8 = 1;
pub const GROUP1_REMOVE_GROUP: u8 = 2;
pub const GROUP1_ADD_GROUP_MEMBERS: u8 = 3;
pub const GROUP1_REMOVE_GROUP_MEMBERS: u8 = 4;
pub const GROUP1_REMOVE_GROUP_ROLES: u8 = 5;
pub const GROUP1_REMOVE_GROUP_MEMBER_ROLES: u8 = 6;

/// Activity inputs for `Group1`.
pub struct ActivityInputGroup1;
impl ActivityInputGroup1 {
    pub fn activity_group_add() -> Vec<Option<ActionInput>> {
        let map = HashMap::new();
        vec![Some(ActionInput {
            action: ActionInputType::DaoAction(DaoActionIdent::GroupAdd),
            values: UserInput::Map(map),
        })]
    }
    pub fn activity_group_remove() -> Vec<Option<ActionInput>> {
        let map = HashMap::new();
        vec![Some(ActionInput {
            action: ActionInputType::DaoAction(DaoActionIdent::GroupRemove),
            values: UserInput::Map(map),
        })]
    }
    pub fn activity_group_add_members() -> Vec<Option<ActionInput>> {
        let map = HashMap::new();
        vec![Some(ActionInput {
            action: ActionInputType::DaoAction(DaoActionIdent::GroupAddMembers),
            values: UserInput::Map(map),
        })]
    }
    pub fn activity_group_remove_members() -> Vec<Option<ActionInput>> {
        let map = HashMap::new();
        vec![Some(ActionInput {
            action: ActionInputType::DaoAction(DaoActionIdent::GroupRemoveMembers),
            values: UserInput::Map(map),
        })]
    }
    pub fn activity_group_remove_roles() -> Vec<Option<ActionInput>> {
        let map = HashMap::new();
        vec![Some(ActionInput {
            action: ActionInputType::DaoAction(DaoActionIdent::GroupRemoveRoles),
            values: UserInput::Map(map),
        })]
    }
    pub fn activity_group_remove_member_roles() -> Vec<Option<ActionInput>> {
        let map = HashMap::new();
        vec![Some(ActionInput {
            action: ActionInputType::DaoAction(DaoActionIdent::GroupRemoveMemberRoles),
            values: UserInput::Map(map),
        })]
    }
    pub fn propose_settings_group_add(
        group_name: &str,
        group_leader: &str,
        members: Vec<&str>,
        role: Option<&str>,
        members_with_role: Vec<&str>,
    ) -> Option<HashMap<String, Value>> {
        let mut map = HashMap::new();
        if let Some(role) = role {
            map.insert(
                "member_roles.0.name".to_string(),
                Value::String(role.into()),
            );
            for (i, member) in members_with_role.into_iter().enumerate() {
                let key = format!("member_roles.0.members.{}", i);
                map.insert(key, Value::String(member.into()));
            }
        }
        map.insert(
            "settings.name".to_string(),
            Value::String(group_name.into()),
        );
        map.insert(
            "settings.leader".to_string(),
            Value::String(group_leader.into()),
        );
        for (i, member) in members.into_iter().enumerate() {
            let key = format!("members.{}.account_id", i);
            map.insert(key, Value::String(member.into()));
        }
        Some(map)
    }
    pub fn propose_settings_group_remove(group_id: u64) -> Option<HashMap<String, Value>> {
        let mut map = HashMap::new();
        map.insert("id".to_string(), Value::U64(group_id));
        Some(map)
    }
    pub fn propose_settings_group_add_members(
        group_id: u64,
        members: Vec<&str>,
        member_roles: Vec<(&str, Vec<&str>)>,
    ) -> Option<HashMap<String, Value>> {
        let mut map = HashMap::new();
        map.insert("id".to_string(), Value::U64(group_id));
        for (i, member) in members.into_iter().enumerate() {
            let key = format!("members.{}.account_id", i);
            map.insert(key, Value::String(member.into()));
        }
        for (i, (name, members)) in member_roles.into_iter().enumerate() {
            let key = format!("member_roles.{}.name", i);
            map.insert(key, Value::String(name.into()));
            for (j, member) in members.into_iter().enumerate() {
                let key = format!("member_roles.{}.members.{}", i, j);
                map.insert(key, Value::String(member.into()));
            }
        }
        Some(map)
    }
    pub fn propose_settings_group_remove_members(
        group_id: u64,
        members: Vec<&str>,
    ) -> Option<HashMap<String, Value>> {
        let mut map = HashMap::new();
        map.insert("id".to_string(), Value::U64(group_id));
        for (i, member) in members.into_iter().enumerate() {
            let key = format!("members.{}", i);
            map.insert(key, Value::String(member.into()));
        }
        Some(map)
    }
    pub fn propose_settings_group_remove_roles(
        group_id: u64,
        role_ids: Vec<u64>,
    ) -> Option<HashMap<String, Value>> {
        let mut map = HashMap::new();
        map.insert("id".to_string(), Value::U64(group_id));
        map.insert("role_ids".into(), Value::VecU64(role_ids));
        Some(map)
    }
    pub fn propose_settings_group_remove_member_roles(
        group_id: u64,
        member_roles: Vec<(&str, Vec<&str>)>,
    ) -> Option<HashMap<String, Value>> {
        let mut map = HashMap::new();
        map.insert("id".to_string(), Value::U64(group_id));
        for (i, (name, members)) in member_roles.into_iter().enumerate() {
            let key = format!("member_roles.{}.name", i);
            map.insert(key, Value::String(name.into()));
            for (j, member) in members.into_iter().enumerate() {
                let key = format!("member_roles.{}.members.{}", i, j);
                map.insert(key, Value::String(member.into()));
            }
        }
        Some(map)
    }
}
