use serde_json::json;
use workspaces::{network::DevAccountDeployer, Contract, DevNetwork, Worker};

use crate::utils::{get_staking_wasm, outcome_pretty};

pub async fn init_staking<T>(worker: &Worker<T>) -> anyhow::Result<Contract>
where
    T: DevNetwork,
{
    let staking_blob_path = get_staking_wasm();
    let staking = worker
        .dev_deploy(&std::fs::read(staking_blob_path)?)
        .await?;
    let args = json!({}).to_string().into_bytes();
    let outcome = staking
        .call(&worker, "new")
        .args(args)
        .max_gas()
        .transact()
        .await?;
    outcome_pretty::<()>("staking init", &outcome);
    assert!(outcome.is_success(), "staking init failed");
    Ok(staking)
}
