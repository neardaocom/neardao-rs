use serde_json::json;
use workspaces::{network::DevAccountDeployer, Account, AccountId, Contract, DevNetwork, Worker};

use crate::utils::{parse_view_result, view_outcome_pretty};

use super::types::init::GroupOutput;

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
