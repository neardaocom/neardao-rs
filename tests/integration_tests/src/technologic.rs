//! Test - playground to verify some implementations work in production environment.

use serde_json::json;
use types::{PercentInput, PercentResult};
use workspaces::network::DevAccountDeployer;

use crate::utils::{
    generate_random_strings, outcome_pretty, parse_call_outcome, view_outcome_pretty,
};

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

#[tokio::test]
async fn calculate_percents() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let wasm_blob = workspaces::compile_project("./../mocks/playground_contract").await?;
    let contract = worker.dev_deploy(&wasm_blob).await?;
    println!("contact id: {}", contract.id());
    let (vote_inputs, expected_results) = generate_data();
    let args = json!({
        "values" : vote_inputs,
    })
    .to_string()
    .into_bytes();
    let outcome = contract
        .call(&worker, "calc_votes")
        .args(args)
        .max_gas()
        .transact()
        .await?;
    let results = parse_call_outcome::<Vec<PercentResult>>(&outcome)
        .expect("failed to parse calc_vote results");
    let mut invalid_results = vec![];
    for (i, result) in results.into_iter().enumerate() {
        if result != expected_results[i] {
            invalid_results.push((result, expected_results[i]));
        }
    }
    assert!(
        invalid_results.is_empty(),
        "invalid vote results: {:#?}",
        invalid_results
    );
    Ok(())
}

macro_rules! generate_vote_data {
    ($data_input:expr, $data_expected:expr, $total_value:expr, $value:expr, $decimals:expr, $expected_percents:expr) => {
        let input = PercentInput::new($total_value.into(), $value.into(), $decimals);
        let expected = PercentResult::from_input(&input, $expected_percents);
        $data_input.push(input);
        $data_expected.push(expected);
    };
}

fn generate_data() -> (Vec<PercentInput>, Vec<PercentResult>) {
    let mut data_input = vec![];
    let mut data_expected = vec![];

    generate_vote_data!(data_input, data_expected, 1, 1, 0, 100);
    generate_vote_data!(data_input, data_expected, 1, 1, 24, 100);
    generate_vote_data!(data_input, data_expected, 1, 0, 0, 0);
    generate_vote_data!(data_input, data_expected, 1, 0, 24, 0);
    generate_vote_data!(data_input, data_expected, 400, 100, 0, 25);
    generate_vote_data!(data_input, data_expected, 400, 100, 24, 25);
    generate_vote_data!(data_input, data_expected, 50_000_000, 0, 0, 0);
    generate_vote_data!(data_input, data_expected, 50_000_000, 0, 24, 0);
    generate_vote_data!(data_input, data_expected, 50_000_000, 249_999, 0, 0);
    generate_vote_data!(data_input, data_expected, 50_000_000, 249_999, 24, 0);
    generate_vote_data!(data_input, data_expected, 50_000_000, 250_000, 24, 1);
    generate_vote_data!(data_input, data_expected, 50_000_000, 250_000, 24, 1);
    generate_vote_data!(data_input, data_expected, 50_000_000, 500_000, 0, 1);
    generate_vote_data!(data_input, data_expected, 50_000_000, 500_000, 24, 1);
    generate_vote_data!(data_input, data_expected, 50_000_000, 10_000_000, 0, 20);
    generate_vote_data!(data_input, data_expected, 50_000_000, 10_000_000, 24, 20);
    generate_vote_data!(data_input, data_expected, 50_000_000, 49_200_000, 0, 98);
    generate_vote_data!(data_input, data_expected, 50_000_000, 49_200_000, 24, 98);
    generate_vote_data!(data_input, data_expected, 50_000_000, 49_500_000, 0, 99);
    generate_vote_data!(data_input, data_expected, 50_000_000, 49_500_000, 24, 99);
    generate_vote_data!(data_input, data_expected, 1_000_000_000, 0, 0, 0);
    generate_vote_data!(data_input, data_expected, 1_000_000_000, 0, 0, 0);
    generate_vote_data!(data_input, data_expected, 1_000_000_000, 4_999_999, 0, 0);
    generate_vote_data!(data_input, data_expected, 1_000_000_000, 4_999_999, 24, 0);
    generate_vote_data!(data_input, data_expected, 1_000_000_000, 5_000_000, 0, 1);
    generate_vote_data!(data_input, data_expected, 1_000_000_000, 5_000_000, 24, 1);
    generate_vote_data!(data_input, data_expected, 1_000_000_000, 10_000_000, 0, 1);
    generate_vote_data!(data_input, data_expected, 1_000_000_000, 10_000_000, 24, 1);
    generate_vote_data!(
        data_input,
        data_expected,
        1_000_000_000,
        1_000_000_000,
        0,
        100
    );
    generate_vote_data!(
        data_input,
        data_expected,
        1_000_000_000,
        1_000_000_000,
        24,
        100
    );
    // 10 bilion is max. that does not overflow using u128.
    generate_vote_data!(data_input, data_expected, 10_000_000_000, 0, 0, 0);
    generate_vote_data!(data_input, data_expected, 10_000_000_000, 0, 24, 0);
    generate_vote_data!(data_input, data_expected, 10_000_000_000, 49_999_999, 0, 0);
    generate_vote_data!(data_input, data_expected, 10_000_000_000, 49_999_999, 24, 0);
    generate_vote_data!(data_input, data_expected, 10_000_000_000, 50_000_000, 0, 1);
    generate_vote_data!(data_input, data_expected, 10_000_000_000, 50_000_000, 24, 1);
    generate_vote_data!(
        data_input,
        data_expected,
        10_000_000_000,
        10_000_000_000,
        0,
        100
    );
    generate_vote_data!(
        data_input,
        data_expected,
        10_000_000_000,
        10_000_000_000,
        24,
        100
    );
    (data_input, data_expected)
}
