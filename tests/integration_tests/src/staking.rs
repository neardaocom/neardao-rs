use anyhow::Result;
use near_sdk::{env, json_types::U128, AccountId, ONE_NEAR, ONE_YOCTO};
use serde::{Deserialize, Serialize};
use serde_json::json;
use workspaces::network::DevAccountDeployer;

use crate::utils::{get_fungible_token, get_staking, outcome_pretty, view_outcome_pretty};

const VIEW_METHOD_GET_USER: &str = "dao_get_user";
const VIEW_METHOD_FT_TOTAL_SUPPLY: &str = "dao_ft_total_supply";
const VIEW_METHOD_FT_BALANCE_OF: &str = "dao_ft_balance_of";

const MIN_STORAGE: u128 = 945;

/// Staking user structure
#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct User {
    /// Amount of staked vote token.
    pub vote_amount: u128,
    /// List of delegations to other accounts.
    /// Invariant: Sum of all delegations <= `self.vote_amount`.
    pub delegated_amounts: Vec<(AccountId, u128)>,
    /// Total delegated amount to this user by others.
    pub delegated_vote_amount: u128,
    /// List of users whom delegated their tokens to this user.
    pub delegators: Vec<AccountId>,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct StorageBalance {
    total: String,
    available: String,
}

#[tokio::test]
async fn staking_full_scenario() -> Result<()> {
    let worker = workspaces::sandbox().await?;
    let staking_blob_path = &get_staking();
    let staking = worker
        .dev_deploy(&std::fs::read(staking_blob_path)?)
        .await?;
    let wasm_blob_dao = workspaces::compile_project("./../mocks/staking_dao").await?;
    let dao = worker.dev_deploy(&wasm_blob_dao).await?;
    let token_blob_path = &get_fungible_token();
    let token = worker.dev_deploy(&std::fs::read(token_blob_path)?).await?;
    let registrar = worker.dev_create_account().await?;
    let token_holder = worker.dev_create_account().await?;
    let delegate_1 = worker.dev_create_account().await?;
    let delegate_2 = worker.dev_create_account().await?;

    // Staking init.
    let args = json!({
        "registrar_id" : registrar.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = staking
        .call(&worker, "new")
        .args(args)
        .max_gas()
        .transact()
        .await?;
    outcome_pretty("staking init", outcome);

    // Dao init.
    let args = json!({
        "staking_id" : staking.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = dao
        .call(&worker, "new")
        .args(args)
        .max_gas()
        .transact()
        .await?;

    assert!(outcome.is_success());
    outcome_pretty("dao init", outcome);

    // FT init.
    let args = json!({
        "owner_id" : token_holder.id(),
        "total_supply": U128::from(1_000_000_000)
    })
    .to_string()
    .into_bytes();

    let outcome = token
        .call(&worker, "new_default_meta")
        .args(args)
        .max_gas()
        .transact()
        .await?;
    assert!(outcome.is_success());
    outcome_pretty("fungible token init", outcome);

    // Check on FT.
    let args = json!({
        "account_id": token_holder.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = token.view(&worker, "ft_balance_of", args).await?;
    view_outcome_pretty::<String>("ft_balance_of token_holder check", outcome);

    // Register new dao by registrar.
    let args = json!({
        "dao_id" : dao.id(),
        "vote_token_id": token.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = registrar
        .call(&worker, staking.id(), "register_new_dao")
        .args(args)
        .max_gas()
        .transact()
        .await?;
    assert!(outcome.is_success());
    outcome_pretty("register new dao by registrar", outcome);

    // Storage deposit token_holder in staking.
    let args = json!({}).to_string().into_bytes();
    let outcome = token_holder
        .call(&worker, staking.id(), "storage_deposit")
        .args(args)
        .max_gas()
        .deposit(MIN_STORAGE * env::storage_byte_cost())
        .transact()
        .await?;
    assert!(outcome.is_success());
    outcome_pretty("storage deposit token_holder in staking", outcome);

    // Check storage balance.
    let args = json!({
        "account_id": token_holder.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = staking.view(&worker, "storage_balance_of", args).await?;
    view_outcome_pretty::<StorageBalance>("storage balance check", outcome);

    // Storage deposit delegate_1 in staking.
    let args = json!({}).to_string().into_bytes();
    let outcome = delegate_1
        .call(&worker, staking.id(), "storage_deposit")
        .args(args)
        .max_gas()
        .deposit(MIN_STORAGE * env::storage_byte_cost())
        .transact()
        .await?;
    assert!(outcome.is_success());
    outcome_pretty("storage deposit delegate_1 in staking", outcome);

    // Storage deposit delegate_2 in staking.
    let args = json!({}).to_string().into_bytes();
    let outcome = delegate_2
        .call(&worker, staking.id(), "storage_deposit")
        .args(args)
        .max_gas()
        .deposit(MIN_STORAGE * env::storage_byte_cost())
        .transact()
        .await?;
    assert!(outcome.is_success());
    outcome_pretty("storage deposit delegate_2 in staking", outcome);

    // Storage deposit staking in fungible_token.
    let args = json!({
        "account_id": staking.id()
    })
    .to_string()
    .into_bytes();
    let outcome = token_holder
        .call(&worker, token.id(), "storage_deposit")
        .args(args)
        .max_gas()
        .deposit(ONE_NEAR)
        .transact()
        .await?;
    assert!(outcome.is_success());
    outcome_pretty("storage deposit staking in fungible_token", outcome);

    // Register token_holder in dao.
    let args = json!({
        "dao_id" : dao.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = token_holder
        .call(&worker, staking.id(), "register_in_dao")
        .args(args)
        .max_gas()
        .transact()
        .await?;
    assert!(outcome.is_success());
    outcome_pretty("register token_holder in dao", outcome);

    // Check storage balance.
    let args = json!({
        "account_id": token_holder.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = staking.view(&worker, "storage_balance_of", args).await?;
    view_outcome_pretty::<StorageBalance>("storage balance check", outcome);

    // Register delegate_1 in dao.
    let args = json!({
        "dao_id" : dao.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = delegate_1
        .call(&worker, staking.id(), "register_in_dao")
        .args(args)
        .max_gas()
        .transact()
        .await?;
    assert!(outcome.is_success());
    outcome_pretty("register delegate_1 in dao", outcome);

    // Register delegate_2 in dao.
    let args = json!({
        "dao_id" : dao.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = delegate_2
        .call(&worker, staking.id(), "register_in_dao")
        .args(args)
        .max_gas()
        .transact()
        .await?;
    assert!(outcome.is_success());
    outcome_pretty("register delegate_2 in dao", outcome);

    // Transfer token to staking.
    let transfer_info = format!("{{\"dao_id\":\"{}\"}}", dao.id());
    let amount = 2_000 + 1_000;
    let args = json!({
        "receiver_id": staking.id(),
        "amount": U128::from(amount),
        "memo": null,
        "msg": transfer_info,
    })
    .to_string()
    .into_bytes();
    let outcome = token_holder
        .call(&worker, token.id(), "ft_transfer_call")
        .args(args)
        .max_gas()
        .deposit(ONE_YOCTO)
        .transact()
        .await?;
    assert!(outcome.is_success());
    outcome_pretty("transfer token to staking", outcome);

    // Check transfer.
    let args = json!({
        "dao_id": dao.id(),
        "account_id": token_holder.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = staking.view(&worker, VIEW_METHOD_GET_USER, args).await?;
    view_outcome_pretty::<User>("transfer check", outcome);

    // Check on FT
    let args = json!({
        "account_id": token_holder.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = token.view(&worker, "ft_balance_of", args).await?;
    view_outcome_pretty::<String>("ft_balance_of token_holder check", outcome);

    // Delegate 1000 ft owned to delegate_1.
    let args = json!({
        "dao_id": dao.id(),
        "delegate_id": delegate_1.id(),
        "amount": U128::from(1_000),
    })
    .to_string()
    .into_bytes();
    let outcome = token_holder
        .call(&worker, staking.id(), "delegate_owned")
        .args(args)
        .max_gas()
        .transact()
        .await?;
    assert!(outcome.is_success());
    outcome_pretty("delegate 1000 ft owned to delegate_1", outcome);

    // Check storage balance.
    let args = json!({
        "account_id": token_holder.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = staking.view(&worker, "storage_balance_of", args).await?;
    view_outcome_pretty::<StorageBalance>("storage balance check", outcome);

    // Delegate 1000 ft owned to delegate_2.
    let args = json!({
        "dao_id": dao.id(),
        "delegate_id": delegate_2.id(),
        "amount": U128::from(1_000),
    })
    .to_string()
    .into_bytes();
    let outcome = token_holder
        .call(&worker, staking.id(), "delegate_owned")
        .args(args)
        .max_gas()
        .transact()
        .await?;
    assert!(outcome.is_success());
    outcome_pretty("delegate 1000 ft owned to delegate_2", outcome);

    // Check storage balance.
    let args = json!({
        "account_id": token_holder.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = staking.view(&worker, "storage_balance_of", args).await?;
    view_outcome_pretty::<StorageBalance>("storage balance check", outcome);

    // Check delegations.
    let args = json!({
        "dao_id": dao.id(),
        "account_id": token_holder.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = staking.view(&worker, VIEW_METHOD_GET_USER, args).await?;
    view_outcome_pretty::<User>("delegation owned check", outcome);

    // Delegate delegated by delegate_1 to delegate_2.
    let args = json!({
        "dao_id": dao.id(),
        "delegate_id": delegate_2.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = delegate_1
        .call(&worker, staking.id(), "delegate")
        .args(args)
        .max_gas()
        .transact()
        .await?;
    assert!(outcome.is_success());
    outcome_pretty(
        "delegate delegated tokens from delegate_1 to delegate_2",
        outcome,
    );

    // Check storage balance.
    let args = json!({
        "account_id": token_holder.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = staking.view(&worker, "storage_balance_of", args).await?;
    view_outcome_pretty::<StorageBalance>("storage balance check", outcome);

    // Check delegation delegated.
    let args = json!({
        "dao_id": dao.id(),
        "account_id": token_holder.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = staking.view(&worker, VIEW_METHOD_GET_USER, args).await?;
    view_outcome_pretty::<User>("delegation delegated check", outcome);

    // Undelegate 500 ft from delegate_2.
    let args = json!({
        "dao_id": dao.id(),
        "delegate_id": delegate_2.id(),
        "amount": U128::from(500),
    })
    .to_string()
    .into_bytes();
    let outcome = token_holder
        .call(&worker, staking.id(), "undelegate")
        .args(args)
        .max_gas()
        .transact()
        .await?;
    assert!(outcome.is_success());
    outcome_pretty("undelegate 500 ft from delegate_2", outcome);

    // Check undelegation.
    let args = json!({
        "dao_id": dao.id(),
        "account_id": token_holder.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = staking.view(&worker, VIEW_METHOD_GET_USER, args).await?;
    view_outcome_pretty::<User>("undelegation check", outcome);

    // Withdraw 500 ft.
    let args = json!({
        "dao_id": dao.id(),
        "amount": U128::from(500),
    })
    .to_string()
    .into_bytes();
    let outcome = token_holder
        .call(&worker, staking.id(), "withdraw")
        .args(args)
        .max_gas()
        .transact()
        .await?;
    assert!(outcome.is_success());
    outcome_pretty("withdraw 500 ft", outcome);

    // Withdraw check.
    let args = json!({
        "dao_id": dao.id(),
        "account_id": token_holder.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = staking.view(&worker, VIEW_METHOD_GET_USER, args).await?;
    view_outcome_pretty::<User>("withdraw check", outcome);

    // Undelegate rest - 1500 ft.
    let args = json!({
        "dao_id": dao.id(),
        "delegate_id": delegate_2.id(),
        "amount": U128::from(1_500),
    })
    .to_string()
    .into_bytes();
    let outcome = token_holder
        .call(&worker, staking.id(), "undelegate")
        .args(args)
        .max_gas()
        .transact()
        .await?;
    assert!(outcome.is_success());
    outcome_pretty("undelegate rest - 1500 ft", outcome);

    // Check storage balance.
    let args = json!({
        "account_id": token_holder.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = staking.view(&worker, "storage_balance_of", args).await?;
    view_outcome_pretty::<StorageBalance>("storage balance check", outcome);

    // Check undelegation.
    let args = json!({
        "dao_id": dao.id(),
        "account_id": token_holder.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = staking.view(&worker, VIEW_METHOD_GET_USER, args).await?;
    view_outcome_pretty::<User>("undelegation check", outcome);

    // Withdraw rest - 1500 + 1000 ft.
    let args = json!({
        "dao_id": dao.id(),
        "amount": U128::from(1_500 + 1_000),
    })
    .to_string()
    .into_bytes();
    let outcome = token_holder
        .call(&worker, staking.id(), "withdraw")
        .args(args)
        .max_gas()
        .transact()
        .await?;
    assert!(outcome.is_success());
    outcome_pretty("withdraw rest - 1500 + 1000 ft", outcome);

    // Withdraw check.
    let args = json!({
        "dao_id": dao.id(),
        "account_id": token_holder.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = staking.view(&worker, VIEW_METHOD_GET_USER, args).await?;
    view_outcome_pretty::<User>("withdraw check", outcome);

    // Check on FT
    let args = json!({
        "account_id": token_holder.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = token.view(&worker, "ft_balance_of", args).await?;
    view_outcome_pretty::<String>("ft_balance_of token_holder check", outcome);

    // Unregister in dao.
    let args = json!({
        "dao_id": dao.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = token_holder
        .call(&worker, staking.id(), "unregister_in_dao")
        .args(args)
        .max_gas()
        .transact()
        .await?;
    assert!(outcome.is_success());
    outcome_pretty("unregister token_holder in dao", outcome);

    // Check storage balance.
    let args = json!({
        "account_id": token_holder.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = staking.view(&worker, "storage_balance_of", args).await?;
    view_outcome_pretty::<StorageBalance>("storage balance check", outcome);

    // Storage unregister in staking.
    let args = json!({}).to_string().into_bytes();
    let outcome = token_holder
        .call(&worker, staking.id(), "storage_unregister")
        .args(args)
        .max_gas()
        .deposit(ONE_YOCTO)
        .transact()
        .await?;
    assert!(outcome.is_success());
    outcome_pretty("storage unregister token_holder in staking", outcome);

    // Storage unregister token_holder check.
    let args = json!({
        "account_id": token_holder.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = staking.view(&worker, "storage_balance_of", args).await?;
    view_outcome_pretty::<String>("storage unregister token_holder in check", outcome);

    Ok(())
}

#[tokio::test]
#[should_panic]
/// Withdraw amount (1500) > vote_amount (3000) - delegated amount (2000).
async fn staking_withdraw_panic() {
    let worker = workspaces::sandbox().await.unwrap();
    let staking_blob_path = &get_staking();
    let staking = worker
        .dev_deploy(&std::fs::read(staking_blob_path).unwrap())
        .await
        .unwrap();
    let wasm_blob_dao = workspaces::compile_project("./../mocks/staking_dao")
        .await
        .unwrap();
    let dao = worker.dev_deploy(&wasm_blob_dao).await.unwrap();
    let token_blob_path = &get_fungible_token();
    let token = worker
        .dev_deploy(&std::fs::read(token_blob_path).unwrap())
        .await
        .unwrap();
    let registrar = worker.dev_create_account().await.unwrap();
    let token_holder = worker.dev_create_account().await.unwrap();
    let delegate_1 = worker.dev_create_account().await.unwrap();
    let delegate_2 = worker.dev_create_account().await.unwrap();

    // Staking init.
    let args = json!({
        "registrar_id" : registrar.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = staking
        .call(&worker, "new")
        .args(args)
        .max_gas()
        .transact()
        .await
        .unwrap();
    assert!(outcome.is_success());

    // Dao init.
    let args = json!({
        "staking_id" : staking.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = dao
        .call(&worker, "new")
        .args(args)
        .max_gas()
        .transact()
        .await
        .unwrap();

    assert!(outcome.is_success());

    // FT init.
    let args = json!({
        "owner_id" : token_holder.id(),
        "total_supply": U128::from(1_000_000_000)
    })
    .to_string()
    .into_bytes();

    let outcome = token
        .call(&worker, "new_default_meta")
        .args(args)
        .max_gas()
        .transact()
        .await
        .unwrap();
    assert!(outcome.is_success());

    // Register new dao by registrar.
    let args = json!({
        "dao_id" : dao.id(),
        "vote_token_id": token.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = registrar
        .call(&worker, staking.id(), "register_new_dao")
        .args(args)
        .max_gas()
        .transact()
        .await
        .unwrap();
    assert!(outcome.is_success());

    // Storage deposit token_holder in staking.
    let args = json!({}).to_string().into_bytes();
    let outcome = token_holder
        .call(&worker, staking.id(), "storage_deposit")
        .args(args)
        .max_gas()
        .deposit(ONE_NEAR)
        .transact()
        .await
        .unwrap();
    assert!(outcome.is_success());

    // Storage deposit delegate_1 in staking.
    let args = json!({}).to_string().into_bytes();
    let outcome = delegate_1
        .call(&worker, staking.id(), "storage_deposit")
        .args(args)
        .max_gas()
        .deposit(ONE_NEAR)
        .transact()
        .await
        .unwrap();
    assert!(outcome.is_success());

    // Storage deposit delegate_2 in staking.
    let args = json!({}).to_string().into_bytes();
    let outcome = delegate_2
        .call(&worker, staking.id(), "storage_deposit")
        .args(args)
        .max_gas()
        .deposit(ONE_NEAR)
        .transact()
        .await
        .unwrap();
    assert!(outcome.is_success());

    // Storage deposit staking in fungible_token.
    let args = json!({
        "account_id": staking.id()
    })
    .to_string()
    .into_bytes();
    let outcome = token_holder
        .call(&worker, token.id(), "storage_deposit")
        .args(args)
        .max_gas()
        .deposit(ONE_NEAR)
        .transact()
        .await
        .unwrap();
    assert!(outcome.is_success());

    // Register token_holder in dao.
    let args = json!({
        "dao_id" : dao.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = token_holder
        .call(&worker, staking.id(), "register_in_dao")
        .args(args)
        .max_gas()
        .transact()
        .await
        .unwrap();
    assert!(outcome.is_success());

    // Register delegate_1 in dao.
    let args = json!({
        "dao_id" : dao.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = delegate_1
        .call(&worker, staking.id(), "register_in_dao")
        .args(args)
        .max_gas()
        .transact()
        .await
        .unwrap();
    assert!(outcome.is_success());

    // Register delegate_2 in dao.
    let args = json!({
        "dao_id" : dao.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = delegate_2
        .call(&worker, staking.id(), "register_in_dao")
        .args(args)
        .max_gas()
        .transact()
        .await
        .unwrap();
    assert!(outcome.is_success());

    // Transfer token to staking.
    let transfer_info = format!("{{\"dao_id\":\"{}\"}}", dao.id());
    let amount = 2_000 + 1_000;
    let args = json!({
        "receiver_id": staking.id(),
        "amount": U128::from(amount),
        "memo": null,
        "msg": transfer_info,
    })
    .to_string()
    .into_bytes();
    let outcome = token_holder
        .call(&worker, token.id(), "ft_transfer_call")
        .args(args)
        .max_gas()
        .deposit(ONE_YOCTO)
        .transact()
        .await
        .unwrap();
    assert!(outcome.is_success());

    // Delegate 1000 ft owned to delegate_1.
    let args = json!({
        "dao_id": dao.id(),
        "delegate_id": delegate_1.id(),
        "amount": U128::from(1_000),
    })
    .to_string()
    .into_bytes();
    let outcome = token_holder
        .call(&worker, staking.id(), "delegate_owned")
        .args(args)
        .max_gas()
        .transact()
        .await
        .unwrap();
    assert!(outcome.is_success());

    // Delegate 1000 ft owned to delegate_2.
    let args = json!({
        "dao_id": dao.id(),
        "delegate_id": delegate_2.id(),
        "amount": U128::from(1_000),
    })
    .to_string()
    .into_bytes();
    let outcome = token_holder
        .call(&worker, staking.id(), "delegate_owned")
        .args(args)
        .max_gas()
        .transact()
        .await
        .unwrap();
    assert!(outcome.is_success());

    // Withdraw 1500 when 3000 is deposited and 2000 is delegated
    let args = json!({
        "dao_id": dao.id(),
        "amount": U128::from(1500),
    })
    .to_string()
    .into_bytes();
    token_holder
        .call(&worker, staking.id(), "withdraw")
        .args(args)
        .max_gas()
        .transact()
        .await
        .unwrap();
}
