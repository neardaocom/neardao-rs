//! Activity inputs for all tested DAO workflows.

mod basic_package;
mod bounty;
mod group;
mod group_package;
mod lock;
mod media;
mod reward;
mod skyward;
mod test_optional_actions;
mod trade;

use library::workflow::action::ActionInput;
use serde_json::json;
use workspaces::{Account, AccountId, DevNetwork, Worker};

use crate::utils::outcome_pretty;

pub use basic_package::*;
pub use bounty::*;
pub use group::*;
pub use group_package::*;
pub use lock::*;
pub use media::*;
pub use reward::*;
pub use skyward::*;
pub use test_optional_actions::*;
pub use trade::*;

pub async fn run_activity<T>(
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

pub async fn workflow_finish<T>(
    worker: &Worker<T>,
    caller: &Account,
    dao: &AccountId,
    proposal_id: u32,
    expected_success: bool,
) -> anyhow::Result<()>
where
    T: DevNetwork,
{
    let args = json!({
        "proposal_id": proposal_id,
    })
    .to_string()
    .into_bytes();
    let outcome = caller
        .call(&worker, dao, "workflow_finish")
        .args(args)
        .max_gas()
        .transact()
        .await;
    let result: bool = outcome.unwrap().json().unwrap();

    let msg = format!(
        "dao: running workflow_finish for proposal_id: {}",
        proposal_id,
    );
    println!("{}", msg);
    assert_eq!(result, expected_success);

    Ok(())
}
