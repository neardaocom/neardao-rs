use serde_json::json;
use workspaces::{network::DevAccountDeployer, AccountId, Contract, DevNetwork, Worker};

use crate::utils::{get_staking_wasm, outcome_pretty};

pub(crate) async fn init_staking<T>(
    worker: &Worker<T>,
    registrar_id: &AccountId,
) -> anyhow::Result<Contract>
where
    T: DevNetwork,
{
    let staking_blob_path = get_staking_wasm();
    let staking = worker
        .dev_deploy(&std::fs::read(staking_blob_path)?)
        .await?;
    let args = json!({
        "registrar_id" :registrar_id.as_str(),
    })
    .to_string()
    .into_bytes();
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
