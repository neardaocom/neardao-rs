use serde_json::json;
use workspaces::{network::DevAccountDeployer, Contract, DevNetwork, Worker};

use crate::utils::{get_wnear_wasm, outcome_pretty};

pub(crate) async fn init_wnear<T>(worker: &Worker<T>) -> anyhow::Result<Contract>
where
    T: DevNetwork,
{
    let wnear_blob_path = get_wnear_wasm();
    let wnear = worker.dev_deploy(&std::fs::read(wnear_blob_path)?).await?;
    let args = json!({}).to_string().into_bytes();
    let outcome = wnear
        .call(&worker, "new")
        .args(args)
        .max_gas()
        .transact()
        .await?;
    outcome_pretty("wnear init", &outcome);
    assert!(outcome.is_success(), "wnear init failed");
    Ok(wnear)
}
