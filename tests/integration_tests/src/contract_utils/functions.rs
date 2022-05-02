use near_sdk::{json_types::U128, ONE_NEAR};
use serde_json::json;
use workspaces::{Account, AccountId, Contract, DevNetwork, Worker};

use crate::utils::outcome_pretty;

pub(crate) async fn storage_deposit<T>(
    worker: &Worker<T>,
    caller: &Account,
    contract: &Contract,
    account_id: &AccountId,
) -> anyhow::Result<()>
where
    T: DevNetwork,
{
    let args = json!({ "account_id": account_id }).to_string().into_bytes();
    let outcome = caller
        .call(&worker, contract.id(), "storage_deposit")
        .args(args)
        .max_gas()
        .deposit(ONE_NEAR)
        .transact()
        .await?;
    assert!(outcome.is_success());
    let msg = format!(
        "{} calls storage deposit in {} for account {}",
        caller.id(),
        contract.id(),
        account_id
    );
    outcome_pretty(&msg, &outcome);
    Ok(())
}
