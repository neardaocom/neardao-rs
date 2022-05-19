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
pub const ADMINPACKAGE1_REMOVE_GROUP_MEMBER: u8 = 4;
pub const ADMINPACKAGE1_ROLE_ADD: u8 = 5;
pub const ADMINPACKAGE1_ROLE_REMOVE: u8 = 6;

/// Activity inputs for `AdminPackage1`.
pub struct ActivityInputAdminPkg1;
impl ActivityInputAdminPkg1 {
    pub fn activity_group_add(
        group_name: &str,
        group_leader: &str,
        members: Vec<&str>,
        role: Option<&str>,
        members_with_role: Vec<&str>,
    ) -> Vec<Option<ActionInput>> {
        let mut roles: HashMap<String, Vec<String>> = HashMap::new();
        if let Some(role) = role {
            roles.insert(
                role.to_string(),
                members_with_role
                    .into_iter()
                    .map(|e| e.to_string())
                    .collect(),
            );
        }
        let roles_json =
            serde_json::to_string(&roles).expect("failed to serialize group member roles");
        let mut map = HashMap::new();
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
        map.insert("member_roles".to_string(), Value::String(roles_json));
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
    ) -> Vec<Option<ActionInput>> {
        let mut map = HashMap::new();
        map.insert("id".to_string(), Value::U64(group_id));
        for (i, member) in members.into_iter().enumerate() {
            let key = format!("members.{}.account_id", i);
            map.insert(key, Value::String(member.into()));
            //let key = format!("members.{}.tags", i);  // Not required
            //map.insert(key, Value::VecU64(vec![]));  // Not required
        }
        vec![Some(ActionInput {
            action: ActionInputType::DaoAction(DaoActionIdent::GroupAddMembers),
            values: UserInput::Map(map),
        })]
    }
    pub fn activity_group_remove_member(group_id: u64, member: &str) -> Vec<Option<ActionInput>> {
        let mut map = HashMap::new();
        map.insert("id".to_string(), Value::U64(group_id));
        map.insert("account_id".to_string(), Value::String(member.into()));
        vec![Some(ActionInput {
            action: ActionInputType::DaoAction(DaoActionIdent::GroupRemoveMember),
            values: UserInput::Map(map),
        })]
    }
    pub fn activity_role_add(
        group_id: u64,
        role_id: u64,
        accounts: Vec<&str>,
    ) -> Vec<Option<ActionInput>> {
        let mut map = HashMap::new();
        map.insert("group_id".to_string(), Value::U64(group_id));
        map.insert("role_id".to_string(), Value::U64(role_id));
        for (i, account) in accounts.into_iter().enumerate() {
            let key = format!("account_ids.{}", i);
            map.insert(key, Value::String(account.into()));
        }
        vec![Some(ActionInput {
            action: ActionInputType::DaoAction(DaoActionIdent::UserRoleAdd),
            values: UserInput::Map(map),
        })]
    }
    pub fn activity_role_remove(
        group_id: u64,
        role_id: u64,
        accounts: Vec<&str>,
    ) -> Vec<Option<ActionInput>> {
        let mut map = HashMap::new();
        map.insert("group_id".to_string(), Value::U64(group_id));
        map.insert("role_id".to_string(), Value::U64(role_id));
        for (i, account) in accounts.into_iter().enumerate() {
            let key = format!("account_ids.{}", i);
            map.insert(key, Value::String(account.into()));
        }
        vec![Some(ActionInput {
            action: ActionInputType::DaoAction(DaoActionIdent::UserRoleRemove),
            values: UserInput::Map(map),
        })]
    }
}
