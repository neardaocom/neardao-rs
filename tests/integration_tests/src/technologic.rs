//! Test - playground to verify some implementations work in production environment.

use serde_json::json;
use workspaces::network::DevAccountDeployer;

use crate::utils::{generate_random_strings, outcome_pretty, view_outcome_pretty};

/// Does not collide with 512 entries. Havent tested more.
#[tokio::test]
#[ignore = "Tested with 512 entries - no collision."]
async fn collisions_in_integer_key_structures() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let wasm_blob = workspaces::compile_project("./../mocks/playground_contract").await?;
    let contract = worker.dev_deploy(&wasm_blob).await?;

    let values = generate_random_strings(512, 8);

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

// Contract can hold max hashmap<String, u8> of approx. 32768 entries with 24 len keys.
// Therefore no need for another partitioning proposal vote structure.
#[tokio::test]
#[ignore = "Approx. 32768 entries with 24 account name len is max."]
async fn hashmap_max_size() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let wasm_blob = workspaces::compile_project("./../mocks/playground_contract").await?;
    let contract = worker.dev_deploy(&wasm_blob).await?;

    let accounts_count = 29 * 1024;
    let accounts_len = 24;

    let values: Vec<(String, u8)> = generate_random_strings(accounts_count, accounts_len)
        .into_iter()
        .map(|acc| (acc, 0))
        .collect();

    let args = json!({
        "values" :values,
    })
    .to_string()
    .into_bytes();
    let outcome = contract
        .call(&worker, "add_to_hm_size_test")
        .args(args)
        .max_gas()
        .transact()
        .await?;

    assert!(outcome.is_success());
    outcome_pretty::<()>("add to hm size 1", &outcome);

    let accounts_count = 3 * 1024;
    let accounts_len = 24;

    let values: Vec<(String, u8)> = generate_random_strings(accounts_count, accounts_len)
        .into_iter()
        .map(|acc| (acc, 0))
        .collect();

    let args = json!({
        "values" :values,
    })
    .to_string()
    .into_bytes();
    let outcome = contract
        .call(&worker, "add_to_hm_size_test")
        .args(args)
        .max_gas()
        .transact()
        .await?;

    assert!(outcome.is_success());
    outcome_pretty::<()>("add to hm size 2 - rest", &outcome);

    let args = json!({}).to_string().into_bytes();
    let outcome = contract.view(&worker, "view_hm_size", args).await?;
    view_outcome_pretty::<usize>("hm max size len check", &outcome);

    Ok(())
}
