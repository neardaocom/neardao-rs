use anyhow::Result;
use library::workflow::types::FnCallMetadata;
use serde_json::json;
use workspaces::network::DevAccountDeployer;

#[ignore = "TODO: implement"]
#[tokio::test]
async fn wf_input_bench() -> Result<()> {
    let worker = workspaces::sandbox().await?;
    let wasm_blob = workspaces::compile_project("./../mocks/simple_dao").await?;
    let contract = worker.dev_deploy(&wasm_blob).await?;

    let metadata: Vec<FnCallMetadata> = vec![];
    let args = json!({ "fncall_metadata": metadata })
        .to_string()
        .into_bytes();

    let outcome = contract
        .call(&worker, "new")
        .args(args)
        .max_gas()
        .transact()
        .await?;

    assert!(outcome.is_success());

    Ok(())
}
