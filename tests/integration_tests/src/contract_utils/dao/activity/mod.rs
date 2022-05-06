mod skyward;
mod wf_add;

use library::workflow::action::ActionInput;
use serde_json::json;
use workspaces::{Account, Contract, DevNetwork, Worker};

use crate::utils::outcome_pretty;

pub use skyward::*;
pub use wf_add::*;

pub(crate) async fn run_activity<T>(
    worker: &Worker<T>,
    caller: &Account,
    dao: &Contract,
    proposal_id: u32,
    activity_id: u8,
    actions_inputs: Vec<Option<ActionInput>>,
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
    outcome_pretty::<()>(&msg, &outcome);
    assert!(outcome.is_success(), "dao running activity failed");

    Ok(())
}
