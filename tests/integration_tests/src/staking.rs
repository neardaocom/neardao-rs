use anyhow::Result;
use near_sdk::{json_types::U128, Balance, ONE_NEAR, ONE_YOCTO};
use serde::{Deserialize, Serialize};
use serde_json::json;
use workspaces::{network::DevAccountDeployer, result::ViewResultDetails};

use crate::utils::{
    get_fungible_token_wasm, get_staking_wasm, outcome_pretty, parse_view_result,
    view_outcome_pretty,
};

const VIEW_METHOD_GET_USER: &str = "dao_get_user";
const VIEW_METHOD_FT_TOTAL_SUPPLY: &str = "dao_ft_total_supply";
const VIEW_METHOD_FT_BALANCE_OF: &str = "dao_ft_balance_of";

const MIN_STORAGE: u128 = 945;

const MIN_STORAGE_DEPOSIT: Balance = 2 * 10u128.pow(23);
pub const MIN_REGISTER_DEPOSIT: Balance = 155 * 10u128.pow(21);

/// Staking user structure
#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct User {
    /// Amount of staked vote token.
    pub vote_amount: u128,
    /// List of delegations to other accounts.
    /// Invariant: Sum of all delegations <= `self.vote_amount`.
    pub delegated_amounts: Vec<(workspaces::AccountId, u128)>,
    /// Total delegated amount to this user by others.
    pub delegated_vote_amount: u128,
    /// List of users whom delegated their tokens to this user.
    pub delegators: Vec<workspaces::AccountId>,
}
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct StorageBalance {
    total: String,
    available: String,
}

fn check_ft_balance_of(
    account_id: &workspaces::AccountId,
    outcome: &ViewResultDetails,
    amount: u128,
) {
    let actual_balance: Option<U128> = outcome.json().unwrap();
    assert_eq!(
        amount,
        actual_balance.unwrap_or(U128(0)).0,
        "{}, vote amount diffs",
        account_id.as_str()
    );
}

//() Checks if user result is as expected
fn check_user_result(
    account_id: &workspaces::AccountId,
    outcome: &ViewResultDetails,
    vote_amount: u128,
    delegated_vote_amount: u128,
    delegated_amounts: Vec<(workspaces::AccountId, u128)>,
    delegators: Vec<workspaces::AccountId>,
) {
    let user: User = outcome.json().unwrap();

    assert_eq!(
        user.vote_amount,
        vote_amount,
        "{}, vote amount diffs",
        account_id.as_str()
    );
    assert_eq!(
        user.delegated_vote_amount,
        delegated_vote_amount,
        "{}, delegated vote amount diffs",
        account_id.as_str()
    );
    assert_eq!(
        user.delegated_amounts,
        delegated_amounts,
        "{}, delegated amounts diffs",
        account_id.as_str()
    );
    assert_eq!(
        user.delegators,
        delegators,
        "{}, delegators diffs",
        account_id.as_str()
    );
}

/// Scenario description:
/// Token holder (TH) registers in dao
/// TH deposits 3000 FT
/// TH delegates 1000 to delegate_1 (D1)
/// TH delegates 1000 to delegate_2 (D2)
/// D1 delegates delegated tokens to delegate_2
/// TH undelegates 500 (now from D2)
/// TH withdraws 1000
/// TH undelegates rest - 1500
/// TH withdraws rest - 2000
/// TH, D1 and D2 unregister from DAO
/// Check DAO's storage balance
/// DAO storage_unregister itself
#[tokio::test]
async fn staking_full_scenario() -> Result<()> {
    let worker = workspaces::sandbox().await?;
    let staking_blob_path = &get_staking_wasm();
    let staking = worker
        .dev_deploy(&std::fs::read(staking_blob_path)?)
        .await?;
    let wasm_blob_dao = workspaces::compile_project("./../mocks/staking_dao").await?;
    let dao = worker.dev_deploy(&wasm_blob_dao).await?;
    let token_blob_path = &get_fungible_token_wasm();
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
    outcome_pretty("staking init", &outcome);

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
    outcome_pretty("dao init", &outcome);

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
    outcome_pretty("fungible token init", &outcome);

    // Check on FT.
    let args = json!({
        "account_id": token_holder.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = token.view(&worker, "ft_balance_of", args).await?;
    view_outcome_pretty::<String>("ft_balance_of token_holder check", &outcome);
    check_ft_balance_of(token_holder.id(), &outcome, 1_000_000_000);

    // Storage deposit for DAO in staking.
    let args = json!({
        "account_id":dao.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = registrar
        .call(&worker, staking.id(), "storage_deposit")
        .args(args)
        .max_gas()
        .deposit(MIN_STORAGE_DEPOSIT)
        .transact()
        .await?;
    assert!(outcome.is_success());
    outcome_pretty("storage deposit DAO in staking", &outcome);

    // Check storage balance of DAO before register.
    let args = json!({
        "account_id": dao.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = staking.view(&worker, "storage_balance_of", args).await?;
    view_outcome_pretty::<StorageBalance>("dao storage balance check before register", &outcome);

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
    outcome_pretty("register new dao by registrar", &outcome);

    // Check storage balance of DAO before register.
    let args = json!({
        "account_id": dao.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = staking.view(&worker, "storage_balance_of", args).await?;
    view_outcome_pretty::<StorageBalance>("dao storage balance check after register", &outcome);
    let storage_balance_before = parse_view_result::<StorageBalance>(&outcome).unwrap();

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
    outcome_pretty("storage deposit staking in fungible_token", &outcome);

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
        .deposit(MIN_REGISTER_DEPOSIT)
        .transact()
        .await?;
    assert!(outcome.is_success());
    outcome_pretty("register token_holder in dao", &outcome);

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
        .deposit(MIN_REGISTER_DEPOSIT)
        .transact()
        .await?;
    assert!(outcome.is_success());
    outcome_pretty("register delegate_1 in dao", &outcome);

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
        .deposit(MIN_REGISTER_DEPOSIT)
        .transact()
        .await?;
    assert!(outcome.is_success());
    outcome_pretty("register delegate_2 in dao", &outcome);

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
    outcome_pretty("transfer token to staking", &outcome);

    // Check transfer.
    let args = json!({
        "dao_id": dao.id(),
        "account_id": token_holder.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = staking.view(&worker, VIEW_METHOD_GET_USER, args).await?;
    view_outcome_pretty::<User>("transfer check", &outcome);
    check_user_result(token_holder.id(), &outcome, 3000, 0, vec![], vec![]);

    // Check on FT
    let args = json!({
        "account_id": token_holder.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = token.view(&worker, "ft_balance_of", args).await?;
    view_outcome_pretty::<String>("ft_balance_of token_holder check", &outcome);
    check_ft_balance_of(token_holder.id(), &outcome, 1_000_000_000 - 3_000);

    // View token_holder weight
    let args = json!({
        "account_id": token_holder.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = dao.view(&worker, "get_user_weight", args).await?;
    view_outcome_pretty::<String>("view token_holder weight", &outcome);
    let result = parse_view_result::<U128>(&outcome).unwrap().0;
    assert_eq!(result, 0);

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
    outcome_pretty("delegate 1000 ft owned to delegate_1", &outcome);

    // delegate_1 check
    let args = json!({
        "dao_id": dao.id(),
        "account_id": delegate_1.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = staking.view(&worker, VIEW_METHOD_GET_USER, args).await?;
    view_outcome_pretty::<User>("delegate_1 check", &outcome);
    check_user_result(
        delegate_1.id(),
        &outcome,
        0,
        1000,
        vec![],
        vec![token_holder.id().to_owned()],
    );

    // Check storage balance.
    let args = json!({
        "account_id": dao.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = staking.view(&worker, "storage_balance_of", args).await?;
    view_outcome_pretty::<StorageBalance>("dao storage balance check", &outcome);

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
    outcome_pretty("delegate 1000 ft owned to delegate_2", &outcome);

    // delegate_2 check
    let args = json!({
        "dao_id": dao.id(),
        "account_id": delegate_2.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = staking.view(&worker, VIEW_METHOD_GET_USER, args).await?;
    view_outcome_pretty::<User>("delegate_2 check", &outcome);
    check_user_result(
        delegate_2.id(),
        &outcome,
        0,
        1000,
        vec![],
        vec![token_holder.id().to_owned()],
    );

    // Check storage balance.
    let args = json!({
        "account_id": dao.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = staking.view(&worker, "storage_balance_of", args).await?;
    view_outcome_pretty::<StorageBalance>("dao storage balance check", &outcome);

    // Check token_holder.
    let args = json!({
        "dao_id": dao.id(),
        "account_id": token_holder.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = staking.view(&worker, VIEW_METHOD_GET_USER, args).await?;
    view_outcome_pretty::<User>("token_holder check", &outcome);
    check_user_result(
        token_holder.id(),
        &outcome,
        3000,
        0,
        vec![
            (delegate_1.id().to_owned(), 1000),
            (delegate_2.id().to_owned(), 1000),
        ],
        vec![],
    );

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
        &outcome,
    );

    // delegate_1 check
    let args = json!({
        "dao_id": dao.id(),
        "account_id": delegate_1.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = staking.view(&worker, VIEW_METHOD_GET_USER, args).await?;
    view_outcome_pretty::<User>("delegate_1 check", &outcome);
    check_user_result(delegate_1.id(), &outcome, 0, 0, vec![], vec![]);

    // delegate_2 check
    let args = json!({
        "dao_id": dao.id(),
        "account_id": delegate_2.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = staking.view(&worker, VIEW_METHOD_GET_USER, args).await?;
    view_outcome_pretty::<User>("delegate_2 check", &outcome);
    check_user_result(
        delegate_2.id(),
        &outcome,
        0,
        2000,
        vec![],
        vec![token_holder.id().to_owned()],
    );

    // Check storage balance.
    let args = json!({
        "account_id": dao.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = staking.view(&worker, "storage_balance_of", args).await?;
    view_outcome_pretty::<StorageBalance>("storage balance check", &outcome);

    // Check delegation delegated.
    let args = json!({
        "dao_id": dao.id(),
        "account_id": token_holder.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = staking.view(&worker, VIEW_METHOD_GET_USER, args).await?;
    view_outcome_pretty::<User>("delegation delegated check", &outcome);
    check_user_result(
        token_holder.id(),
        &outcome,
        3000,
        0,
        vec![(delegate_2.id().to_owned(), 2000)],
        vec![],
    );

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
    outcome_pretty("undelegate 500 ft from delegate_2", &outcome);

    // delegate_2 check
    let args = json!({
        "dao_id": dao.id(),
        "account_id": delegate_2.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = staking.view(&worker, VIEW_METHOD_GET_USER, args).await?;
    view_outcome_pretty::<User>("delegate_2 check", &outcome);
    check_user_result(
        delegate_2.id(),
        &outcome,
        0,
        1500,
        vec![],
        vec![token_holder.id().to_owned()],
    );

    // Check token_holder.
    let args = json!({
        "dao_id": dao.id(),
        "account_id": token_holder.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = staking.view(&worker, VIEW_METHOD_GET_USER, args).await?;
    view_outcome_pretty::<User>("token_holder check", &outcome);
    check_user_result(
        token_holder.id(),
        &outcome,
        3000,
        0,
        vec![(delegate_2.id().to_owned(), 1500)],
        vec![],
    );

    // Withdraw 1000 ft.
    let args = json!({
        "dao_id": dao.id(),
        "amount": U128::from(1_000),
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
    outcome_pretty("withdraw 1000 ft", &outcome);

    // Withdraw check.
    let args = json!({
        "dao_id": dao.id(),
        "account_id": token_holder.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = staking.view(&worker, VIEW_METHOD_GET_USER, args).await?;
    view_outcome_pretty::<User>("withdraw check", &outcome);
    check_user_result(
        token_holder.id(),
        &outcome,
        2000,
        0,
        vec![(delegate_2.id().to_owned(), 1500)],
        vec![],
    );

    // Check on FT
    let args = json!({
        "account_id": token_holder.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = token.view(&worker, "ft_balance_of", args).await?;
    view_outcome_pretty::<String>("ft_balance_of token_holder check", &outcome);
    check_ft_balance_of(token_holder.id(), &outcome, 1_000_000_000 - 2_000);

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
    outcome_pretty("undelegate rest - 1500 ft", &outcome);

    // delegate_2 check
    let args = json!({
        "dao_id": dao.id(),
        "account_id": delegate_2.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = staking.view(&worker, VIEW_METHOD_GET_USER, args).await?;
    view_outcome_pretty::<User>("delegate_2 check", &outcome);
    check_user_result(delegate_2.id(), &outcome, 0, 0, vec![], vec![]);

    // Check undelegation.
    let args = json!({
        "dao_id": dao.id(),
        "account_id": token_holder.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = staking.view(&worker, VIEW_METHOD_GET_USER, args).await?;
    view_outcome_pretty::<User>("undelegation check", &outcome);
    check_user_result(token_holder.id(), &outcome, 2000, 0, vec![], vec![]);

    // token_holder token weight check
    let args = json!({
        "account_id": token_holder.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = dao.view(&worker, "get_user_weight", args).await?;
    view_outcome_pretty::<String>("token_holder weight check", &outcome);
    let result = parse_view_result::<U128>(&outcome).unwrap().0;
    assert_eq!(result, 0);

    // Check storage balance.
    let args = json!({
        "account_id": dao.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = staking.view(&worker, "storage_balance_of", args).await?;
    view_outcome_pretty::<StorageBalance>("dao storage balance check", &outcome);

    // Withdraw rest - 1500 + 500 ft.
    let args = json!({
        "dao_id": dao.id(),
        "amount": U128::from(1_500 + 500),
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
    outcome_pretty("withdraw rest - 1500 + 500 ft", &outcome);

    // Withdraw check.
    let args = json!({
        "dao_id": dao.id(),
        "account_id": token_holder.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = staking.view(&worker, VIEW_METHOD_GET_USER, args).await?;
    view_outcome_pretty::<User>("withdraw check", &outcome);
    check_user_result(token_holder.id(), &outcome, 0, 0, vec![], vec![]);

    // Check on FT
    let args = json!({
        "account_id": token_holder.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = token.view(&worker, "ft_balance_of", args).await?;
    view_outcome_pretty::<String>("ft_balance_of token_holder check", &outcome);
    check_ft_balance_of(token_holder.id(), &outcome, 1_000_000_000);

    // Unregister token_holder in dao.
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
    outcome_pretty("unregister token_holder in dao", &outcome);

    let args = json!({
        "dao_id": dao.id(),
        "account_id": token_holder.id(),
    })
    .to_string()
    .into_bytes();
    assert!(staking
        .view(&worker, VIEW_METHOD_GET_USER, args)
        .await
        .is_err());

    // Unregister delegate_1 in dao.
    let args = json!({
        "dao_id": dao.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = delegate_1
        .call(&worker, staking.id(), "unregister_in_dao")
        .args(args)
        .max_gas()
        .transact()
        .await?;
    assert!(outcome.is_success());
    outcome_pretty("unregister delegate_1 in dao", &outcome);

    let args = json!({
        "dao_id": dao.id(),
        "account_id": delegate_1.id(),
    })
    .to_string()
    .into_bytes();
    assert!(staking
        .view(&worker, VIEW_METHOD_GET_USER, args)
        .await
        .is_err());

    // Unregister delegate_2 in dao.
    let args = json!({
        "dao_id": dao.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = delegate_2
        .call(&worker, staking.id(), "unregister_in_dao")
        .args(args)
        .max_gas()
        .transact()
        .await?;
    assert!(outcome.is_success());
    outcome_pretty("unregister delegate_2 in dao", &outcome);

    let args = json!({
        "dao_id": dao.id(),
        "account_id": delegate_2.id(),
    })
    .to_string()
    .into_bytes();
    assert!(staking
        .view(&worker, VIEW_METHOD_GET_USER, args)
        .await
        .is_err());

    // Check storage balance after unregistering all 3 users.
    let args = json!({
        "account_id": dao.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = staking.view(&worker, "storage_balance_of", args).await?;
    view_outcome_pretty::<StorageBalance>(
        "dao storage balance check after unregister all",
        &outcome,
    );
    let storage_balance_after = parse_view_result::<StorageBalance>(&outcome).unwrap();
    assert_eq!(
        storage_balance_before, storage_balance_after,
        "{}",
        "Storage balance does not equal."
    );

    // Storage unregister DAO.
    let args = json!({}).to_string().into_bytes();
    let outcome = dao
        .as_account()
        .call(&worker, staking.id(), "storage_unregister")
        .args(args)
        .max_gas()
        .deposit(ONE_YOCTO)
        .transact()
        .await?;
    assert!(outcome.is_success());
    outcome_pretty("storage unregister dao", &outcome);

    let args = json!({
        "dao_id": dao.id(),
    })
    .to_string()
    .into_bytes();
    assert!(staking
        .view(&worker, VIEW_METHOD_FT_TOTAL_SUPPLY, args)
        .await
        .is_err());

    Ok(())
}

#[tokio::test]
/// Withdraw amount (1500) > vote_amount (3000) - delegated amount (2000).
async fn staking_withdraw_panic() {
    let worker = workspaces::sandbox().await.unwrap();
    let staking_blob_path = &get_staking_wasm();
    let staking = worker
        .dev_deploy(&std::fs::read(staking_blob_path).unwrap())
        .await
        .unwrap();
    let wasm_blob_dao = workspaces::compile_project("./../mocks/staking_dao")
        .await
        .unwrap();
    let dao = worker.dev_deploy(&wasm_blob_dao).await.unwrap();
    let token_blob_path = &get_fungible_token_wasm();
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

    // Storage deposit for DAO in staking.
    let args = json!({
        "account_id":dao.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = registrar
        .call(&worker, staking.id(), "storage_deposit")
        .args(args)
        .max_gas()
        .deposit(MIN_STORAGE_DEPOSIT)
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
        .deposit(MIN_REGISTER_DEPOSIT)
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
        .deposit(MIN_REGISTER_DEPOSIT)
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
        .deposit(MIN_REGISTER_DEPOSIT)
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
    assert!(token_holder
        .call(&worker, staking.id(), "withdraw")
        .args(args)
        .max_gas()
        .transact()
        .await
        .is_err());
}
