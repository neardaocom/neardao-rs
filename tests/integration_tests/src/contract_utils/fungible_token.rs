use near_sdk::json_types::U128;
use serde_json::json;
use workspaces::{network::DevAccountDeployer, AccountId, Contract, DevNetwork, Worker};

use crate::utils::{get_fungible_token_wasm, outcome_pretty};

pub(crate) async fn init_fungible_token<T>(
    worker: &Worker<T>,
    owner_id: &AccountId,
    total_supply: u128,
) -> anyhow::Result<Contract>
where
    T: DevNetwork,
{
    let token_blob_path = get_fungible_token_wasm();
    let token = worker.dev_deploy(&std::fs::read(token_blob_path)?).await?;

    let args = json!({
        "owner_id" :owner_id,
        "total_supply" : U128(total_supply),
    })
    .to_string()
    .into_bytes();
    let outcome = token
        .call(&worker, "new_default_meta")
        .args(args)
        .max_gas()
        .transact()
        .await?;
    outcome_pretty::<()>("fungible token init", &outcome);
    assert!(outcome.is_success(), "fungible token init failed");
    Ok(token)
}
