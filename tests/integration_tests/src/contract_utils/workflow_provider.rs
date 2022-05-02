use library::workflow::template::Template;
use serde_json::json;
use workspaces::{network::DevAccountDeployer, Contract, DevNetwork, Worker};

use crate::utils::{get_wf_provider_wasm, outcome_pretty};

pub(crate) async fn init_workflow_provider<T>(worker: &Worker<T>) -> anyhow::Result<Contract>
where
    T: DevNetwork,
{
    let provider_blob_path = get_wf_provider_wasm();
    let provider = worker
        .dev_deploy(&std::fs::read(provider_blob_path)?)
        .await?;
    let args = json!({}).to_string().into_bytes();
    let outcome = provider
        .call(&worker, "new")
        .args(args)
        .max_gas()
        .transact()
        .await?;
    outcome_pretty("workflow provider init", &outcome);
    assert!(outcome.is_success(), "workflow provider init failed");
    Ok(provider)
}

fn wf_templates() -> Vec<Template> {
    let mut templates = vec![];

    templates
}
