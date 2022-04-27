//! Test - playground to verify some implementations work in production environment.

use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde_json::json;
use workspaces::network::DevAccountDeployer;

#[tokio::test]
#[ignore = "Works"]
async fn collisions_in_integer_key_structures() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let wasm_blob = workspaces::compile_project("./../mocks/playground_contract").await?;
    let contract = worker.dev_deploy(&wasm_blob).await?;

    let values = generate_random_strings(512);

    // Lookupmap
    let args = json!({
        "values" :values,
    })
    .to_string()
    .into_bytes();
    let outcome = contract
        .call(&worker, "add_to_lm")
        .args(args)
        .max_gas()
        .transact()
        .await?;

    assert!(outcome.is_success());

    // Hashmap
    let args = json!({
        "values" :values,
    })
    .to_string()
    .into_bytes();
    let outcome = contract
        .call(&worker, "add_to_hm")
        .args(args)
        .max_gas()
        .transact()
        .await?;
    assert!(outcome.is_success());

    // Nested
    let args = json!({
        "values" :values,
    })
    .to_string()
    .into_bytes();
    let outcome = contract
        .call(&worker, "add_to_nested")
        .args(args)
        .max_gas()
        .transact()
        .await?;
    assert!(outcome.is_success());

    Ok(())
}

fn generate_random_strings(count: usize) -> Vec<String> {
    assert!(count > 0, "Called with zero");
    let mut vec = Vec::with_capacity(100);
    for _ in 0..count {
        let string: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(8)
            .map(char::from)
            .collect();
        vec.push(string);
    }
    vec
}
