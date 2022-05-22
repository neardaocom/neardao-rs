use library::{types::datatype::Value, workflow::instance::InstanceState};
use serde_json::json;
use workspaces::{AccountId, DevNetwork, Worker};

use crate::{
    types::{Group, GroupMembers, GroupSettings, Roles, UserRoles},
    utils::{parse_view_result, view_group_roles, view_outcome_pretty},
};

use super::view::{
    view_groups, view_user_roles, workflow_instance, workflow_storage, workflow_templates,
};

type GroupOutput = (u16, Group);

pub async fn check_group<T>(
    worker: &Worker<T>,
    dao: &AccountId,
    expected_group_id: u16,
    expected_group_name: &str,
    expected_group_leader: Option<&str>,
    expected_group_parent: u16,
    expected_group_members: Vec<(&str, Vec<u16>)>,
    expected_group_rewards: Vec<(u16, u16)>,
) -> anyhow::Result<()>
where
    T: DevNetwork,
{
    let members = expected_group_members
        .into_iter()
        .map(|(a, t)| (AccountId::try_from(a.to_string()).unwrap(), t))
        .collect();
    let expected = (
        expected_group_id,
        Group {
            settings: GroupSettings {
                name: expected_group_name.to_string(),
                leader: expected_group_leader.map(|l| AccountId::try_from(l.to_string()).unwrap()),
                parent_group: expected_group_parent,
            },
            members: GroupMembers(members),
            rewards: expected_group_rewards,
        },
    );
    internal_check_group(worker, dao, expected).await
}

pub async fn internal_check_group<T>(
    worker: &Worker<T>,
    dao: &AccountId,
    group: GroupOutput,
) -> anyhow::Result<()>
where
    T: DevNetwork,
{
    let args = json!({
        "id": group.0,
    })
    .to_string()
    .into_bytes();
    let outcome = worker.view(&dao, "group", args).await?;
    view_outcome_pretty::<Group>("dao check_group", &outcome);
    let actual = parse_view_result::<Group>(&outcome).expect("Group not found");
    assert_eq!(group.1, actual, "check_group: groups are not equal");
    Ok(())
}

pub async fn check_instance<T>(
    worker: &Worker<T>,
    dao: &AccountId,
    proposal_id: u32,
    expected_activity_id: u8,
    expected_actions_done: u8,
    expected_state: InstanceState,
) -> anyhow::Result<()>
where
    T: DevNetwork,
{
    let instance = workflow_instance(worker, dao, proposal_id)
        .await?
        .expect("failed to get workflow instance");
    dbg!(instance.clone());
    assert_eq!(
        instance.get_current_activity_id(),
        expected_activity_id,
        "check_instance_state: instance activities are not equal",
    );
    assert_eq!(
        instance.actions_done_count(),
        expected_actions_done,
        "check_instance_state: instance actions done are not equal",
    );
    assert_eq!(
        instance.get_state(),
        expected_state,
        "check_instance_state: instance states are not equal",
    );
    Ok(())
}

pub async fn check_wf_templates<T>(
    worker: &Worker<T>,
    dao: &AccountId,
    expected_count: usize,
) -> anyhow::Result<()>
where
    T: DevNetwork,
{
    let templates = workflow_templates(worker, dao).await?;
    assert_eq!(
        templates.len(),
        expected_count,
        "check_wf_templates: dao wf templates count does not match expected",
    );
    Ok(())
}

pub async fn check_wf_storage_values<T>(
    worker: &Worker<T>,
    dao: &AccountId,
    workflow_storage_key: String,
    expected_values: Vec<(String, Value)>,
) -> anyhow::Result<()>
where
    T: DevNetwork,
{
    let storage = workflow_storage(worker, dao, workflow_storage_key)
        .await?
        .expect("check_wf_storage_values: workflow storage not found");

    for value in expected_values {
        assert!(
            storage.contains(&value),
            "check_wf_templates: some of the expected values not found in the workflow storage: {:?}", storage
        );
    }
    Ok(())
}

pub async fn check_group_stats<T>(
    worker: &Worker<T>,
    dao: &AccountId,
    group_name: &str,
    group_leader: Option<&str>,
    group_members: Vec<&str>,
) -> anyhow::Result<()>
where
    T: DevNetwork,
{
    let actual_groups = view_groups(worker, dao).await?;
    let group_members: Vec<AccountId> = group_members
        .into_iter()
        .map(|m| AccountId::try_from(m.to_string()).unwrap())
        .collect();
    let mut found = false;
    for (_, group) in actual_groups.into_iter() {
        if group.settings.name == group_name
            && group.settings.leader.as_ref().map(|s| s.as_str()) == group_leader
            && group_members
                == group
                    .members
                    .0
                    .into_iter()
                    .map(|m| m.0)
                    .collect::<Vec<AccountId>>()
        {
            found = true;
            break;
        }
    }
    let err_msg = format!(
        "Group: {} with leader: {} not found.",
        group_name,
        group_leader.unwrap_or_default()
    );
    assert!(found, "{}", err_msg);
    Ok(())
}

pub async fn check_group_exists<T>(
    worker: &Worker<T>,
    dao: &AccountId,
    group_name: &str,
    should_exist: bool,
) -> anyhow::Result<()>
where
    T: DevNetwork,
{
    let name = group_name.to_string();
    let groups = view_groups(worker, dao).await?;
    let mut found = false;
    for (_, g) in groups {
        if g.settings.name == name {
            found = true;
            break;
        }
    }
    assert!(should_exist == found);

    Ok(())
}

pub async fn check_user_roles<T>(
    worker: &Worker<T>,
    dao: &AccountId,
    account_id: &str,
    expected_roles: Option<&UserRoles>,
) -> anyhow::Result<()>
where
    T: DevNetwork,
{
    let account_id = AccountId::try_from(account_id.to_string()).unwrap();
    let roles = view_user_roles(worker, dao, &account_id).await?;

    assert_eq!(
        roles.map(|r| r.sort()),
        expected_roles.map(|r| r.to_owned().sort()),
        "Roles do not match"
    );

    Ok(())
}

pub async fn check_group_roles<T>(
    worker: &Worker<T>,
    dao: &AccountId,
    group_id: u16,
    expected_group_roles: &Roles,
) -> anyhow::Result<()>
where
    T: DevNetwork,
{
    let roles = view_group_roles(worker, dao, group_id).await?;
    assert_eq!(roles, *expected_group_roles, "Roles do not match");
    Ok(())
}
