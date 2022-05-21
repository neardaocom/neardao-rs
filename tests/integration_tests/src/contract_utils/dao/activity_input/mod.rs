mod admin_package;
mod bounty;
mod reward;
mod skyward;
pub mod test_optional_actions;
mod trade;
mod wf_add;

use library::workflow::action::ActionInput;
use serde_json::json;
use workspaces::{Account, AccountId, Contract, DevNetwork, Worker};

use crate::utils::outcome_pretty;

pub use admin_package::*;
pub use bounty::*;
pub use reward::*;
pub use skyward::*;
pub use trade::*;
pub use wf_add::*;

pub(crate) async fn run_activity<T>(
    worker: &Worker<T>,
    caller: &Account,
    dao: &AccountId,
    proposal_id: u32,
    activity_id: u8,
    actions_inputs: Vec<Option<ActionInput>>,
    expected_success: bool,
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
        .call(&worker, dao, "workflow_run_activity")
        .args(args)
        .max_gas()
        .transact()
        .await;
    let msg = format!(
        "dao: running proposal_id: {} activity_id: {}",
        proposal_id, activity_id
    );
    dbg!(proposal_id);
    dbg!(activity_id);
    dbg!(expected_success);
    println!("{}", msg);
    if expected_success {
        assert!(
            outcome.is_ok() && outcome.as_ref().unwrap().is_success(),
            "dao running activity failed - expected success, result: {:?}",
            outcome
        );
        let msg = format!(
            "proposal proposal_id: {} activity_id: {} outcome",
            proposal_id, activity_id
        );
        outcome_pretty::<()>(&msg, &outcome.unwrap());
    } else {
        assert!(
            outcome.is_err() || outcome.as_ref().unwrap().is_failure(),
            "dao running activity failed - expected failure, result: {:?}",
            outcome
        );
    }

    Ok(())
}
