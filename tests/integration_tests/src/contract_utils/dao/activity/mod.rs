mod skyward;
mod wf_add;

use library::workflow::{action::ActionInput, instance::InstanceState};
use serde_json::json;
use workspaces::{Account, Contract, DevNetwork, Worker};

use crate::{contract_utils::dao::checks::check_instance, utils::outcome_pretty};

pub use wf_add::*;

pub(crate) async fn run_activity<T>(
    worker: &Worker<T>,
    caller: &Account,
    dao: &Contract,
    proposal_id: u32,
    activity_id: u8,
    actions_inputs: Vec<Option<ActionInput>>,
    expected_activity_state: InstanceState,
) -> anyhow::Result<()>
where
    T: DevNetwork,
{
    let args = json!({
        "proposal_id": proposal_id,
        "activity_id": activity_id,
        "actions_inputs": actions_inputs,
    })
    .to_string()
    .into_bytes();
    let outcome = caller
        .call(&worker, dao.id(), "workflow_run_activity")
        .args(args)
        .max_gas()
        .transact()
        .await?;
    let msg = format!(
        "dao: running proposal_id: {} activity_id: {}",
        proposal_id, activity_id
    );
    outcome_pretty(&msg, &outcome);
    assert!(outcome.is_success(), "dao running activity failed");

    check_instance(
        worker,
        dao,
        proposal_id,
        activity_id,
        expected_activity_state,
    )
    .await?;

    Ok(())
}
