use std::collections::HashMap;

use library::{
    types::{activity_input::UserInput, datatype::Value},
    workflow::{
        action::{ActionInput, ActionInputType},
        types::DaoActionIdent,
    },
};

pub const ADMINPACKAGE1_ADD_GROUP: u8 = 1;
pub const ADMINPACKAGE1_REMOVE_GROUP: u8 = 2;
pub const ADMINPACKAGE1_ADD_GROUP_MEMBERS: u8 = 3;
pub const ADMINPACKAGE1_REMOVE_GROUP_MEMBERS: u8 = 4;
pub const ADMINPACKAGE1_REMOVE_GROUP_ROLES: u8 = 5;
pub const ADMINPACKAGE1_REMOVE_GROUP_MEMBER_ROLES: u8 = 6;

/// Activity inputs for `GroupPackage1`.
pub struct ActivityInputGroupPkg1;
impl ActivityInputGroupPkg1 {
    pub fn activity_group_add(
        group_name: &str,
        group_leader: &str,
        members: Vec<&str>,
        role: Option<&str>,
        members_with_role: Vec<&str>,
    ) -> Vec<Option<ActionInput>> {
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
        //map.insert("settings.parent_group".to_string(), Value::U64(0)); // Not required
        for (i, member) in members.into_iter().enumerate() {
            let key = format!("members.{}.account_id", i);
            map.insert(key, Value::String(member.into()));
            //let key = format!("members.{}.tags", i);  // Not required
            //map.insert(key, Value::VecU64(vec![]));  // Not required
        }
        vec![Some(ActionInput {
            action: ActionInputType::DaoAction(DaoActionIdent::GroupAdd),
            values: UserInput::Map(map),
        })]
    }
    pub fn activity_group_remove(group_id: u64) -> Vec<Option<ActionInput>> {
        let mut map = HashMap::new();
        map.insert("id".to_string(), Value::U64(group_id));
        vec![Some(ActionInput {
            action: ActionInputType::DaoAction(DaoActionIdent::GroupRemove),
            values: UserInput::Map(map),
        })]
    }
    pub fn activity_group_add_members(
        group_id: u64,
        members: Vec<&str>,
        member_roles: Vec<(&str, Vec<&str>)>,
    ) -> Vec<Option<ActionInput>> {
        let mut map = HashMap::new();
        map.insert("id".to_string(), Value::U64(group_id));
        for (i, member) in members.into_iter().enumerate() {
            let key = format!("members.{}.account_id", i);
            map.insert(key, Value::String(member.into()));
            //let key = format!("members.{}.tags", i);  // Not required
            //map.insert(key, Value::VecU64(vec![]));  // Not required
        }
        for (i, (name, members)) in member_roles.into_iter().enumerate() {
            let key = format!("member_roles.{}.name", i);
            map.insert(key, Value::String(name.into()));
            for (j, member) in members.into_iter().enumerate() {
                let key = format!("member_roles.{}.members.{}", i, j);
                map.insert(key, Value::String(member.into()));
            }
        }
        vec![Some(ActionInput {
            action: ActionInputType::DaoAction(DaoActionIdent::GroupAddMembers),
            values: UserInput::Map(map),
        })]
    }
    pub fn activity_group_remove_members(
        group_id: u64,
        members: Vec<&str>,
    ) -> Vec<Option<ActionInput>> {
        let mut map = HashMap::new();
        map.insert("id".to_string(), Value::U64(group_id));
        for (i, member) in members.into_iter().enumerate() {
            let key = format!("members.{}", i);
            map.insert(key, Value::String(member.into()));
        }
        vec![Some(ActionInput {
            action: ActionInputType::DaoAction(DaoActionIdent::GroupRemoveMembers),
            values: UserInput::Map(map),
        })]
    }
    pub fn activity_group_remove_roles(
        group_id: u64,
        role_ids: Vec<u64>,
    ) -> Vec<Option<ActionInput>> {
        let mut map = HashMap::new();
        map.insert("id".to_string(), Value::U64(group_id));
        map.insert("role_ids".into(), Value::VecU64(role_ids));
        vec![Some(ActionInput {
            action: ActionInputType::DaoAction(DaoActionIdent::GroupRemoveRoles),
            values: UserInput::Map(map),
        })]
    }
    pub fn activity_group_remove_member_roles(
        group_id: u64,
        member_roles: Vec<(&str, Vec<&str>)>,
    ) -> Vec<Option<ActionInput>> {
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
        vec![Some(ActionInput {
            action: ActionInputType::DaoAction(DaoActionIdent::GroupRemoveMemberRoles),
            values: UserInput::Map(map),
        })]
    }
}
