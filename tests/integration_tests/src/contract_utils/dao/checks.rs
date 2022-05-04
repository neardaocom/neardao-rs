use library::workflow::instance::InstanceState;
use serde_json::json;
use workspaces::{AccountId, Contract, DevNetwork, Worker};

use crate::utils::{parse_view_result, view_outcome_pretty};

use super::{types::init::GroupOutput, views::workflow_instance};

pub(crate) async fn check_group<T>(
    worker: &Worker<T>,
    dao: &Contract,
    expected_group_id: u16,
    expected_group_name: String,
    expected_group_leader: Option<&AccountId>,
    expected_group_parent: u16,
    expected_group_members: Vec<(&AccountId, Vec<u16>)>,
) -> anyhow::Result<()>
where
    T: DevNetwork,
{
    let expected = GroupOutput::from_expected(
        expected_group_id,
        expected_group_name,
        expected_group_leader.map(|l| l.to_owned()),
        expected_group_parent,
        expected_group_members,
    );
    internal_check_group(worker, dao, expected).await
}

pub(crate) async fn internal_check_group<T>(
    worker: &Worker<T>,
    dao_contract: &Contract,
    group: GroupOutput,
) -> anyhow::Result<()>
where
    T: DevNetwork,
{
    let args = json!({
        "id": group.id,
    })
    .to_string()
    .into_bytes();
    let outcome = dao_contract.view(&worker, "group", args).await?;
    view_outcome_pretty::<GroupOutput>("dao check_group", &outcome);
    let actual = parse_view_result::<GroupOutput>(&outcome).expect("Group not found");
    assert_eq!(group, actual, "check_group: groups are not equal");
    Ok(())
}

pub(crate) async fn check_instance<T>(
    worker: &Worker<T>,
    dao: &Contract,
    proposal_id: u32,
    expected_activity_id: u8,
    expected_state: InstanceState,
) -> anyhow::Result<()>
where
    T: DevNetwork,
{
    let instance = workflow_instance(worker, dao, proposal_id)
        .await?
        .expect("failed to get workflow instance");
    assert_eq!(
        instance.current_activity_id, expected_activity_id,
        "check_instance_state: instance activities are not equal"
    );
    assert_eq!(
        instance.state, expected_state,
        "check_instance_state: instance states are not equal"
    );
    Ok(())
}
