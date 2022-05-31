use anyhow::Result;
use near_sdk::{json_types::U128, Balance, ONE_YOCTO};
use serde_json::json;
use workspaces::{network::DevAccountDeployer, result::ViewResultDetails, AccountId};

use crate::{
    types::{InitDistribution, StorageBalance},
    utils::{
        check_dao_user_weight, create_dao_via_factory, create_ft_via_factory, ft_balance_of,
        ft_transfer_call, init_dao_factory, init_ft_factory, init_staking, init_workflow_provider,
        outcome_pretty, staking_check_user, storage_balance_of, storage_deposit,
        DAO_FT_TOTAL_SUPPLY, MAX_GAS,
    },
};

const VIEW_METHOD_GET_USER: &str = "dao_get_user";
const VIEW_METHOD_FT_TOTAL_SUPPLY: &str = "dao_ft_total_supply";
const VIEW_METHOD_FT_BALANCE_OF: &str = "dao_ft_balance_of";

const MIN_STORAGE: u128 = 945;

const MIN_STORAGE_DEPOSIT: Balance = 2 * 10u128.pow(23);
pub const MIN_REGISTER_DEPOSIT: Balance = 155 * 10u128.pow(19);
pub const DECIMALS: u128 = 10u128.pow(24);
pub const STANDARD_FT_STORAGE_DEPOSIT: Balance = 1_250_000_000_000_000_000_000;

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

/// Scenario description:
/// Token holder (TH) registers in dao
/// TH deposits 4000 FT
/// TH delegates 1000 to self
/// TH delegates 1000 to delegate_1 (D1)
/// TH delegates 1000 to delegate_2 (D2)
/// D1 delegates delegated tokens to delegate_2
/// TH undelegates 500 (now from D2)
/// TH withdraws 1000
/// TH undelegates rest from d2 - 1500
/// TH undelegates 1000 from self
/// TH withdraws rest - 3000
/// TH, D1 and D2 unregister from DAO
/// Check DAO's storage balance
/// DAO storage_unregister itself
#[tokio::test]
async fn staking_full_scenario() -> Result<()> {
    let dao_name = "test_dao";
    let ft_name = "dao_token";
    let worker = workspaces::sandbox().await?;
    let registrar = worker.dev_create_account().await?;
    let token_holder = worker.dev_create_account().await?;
    let delegate_1 = worker.dev_create_account().await?;
    let delegate_2 = worker.dev_create_account().await?;
    let ft_factory = init_ft_factory(&worker).await?;
    let factory = init_dao_factory(&worker).await?;
    let dao_account_id = AccountId::try_from(format!("{}.{}", dao_name, factory.id()))
        .expect("invalid dao account id");
    let token_account_id = AccountId::try_from(format!("{}.{}", ft_name, ft_factory.id()))
        .expect("invalid ft account id");
    create_ft_via_factory(
        &worker,
        &ft_factory,
        ft_name,
        dao_account_id.as_str(),
        DAO_FT_TOTAL_SUPPLY,
        24,
        None,
        None,
        vec![InitDistribution {
            account_id: token_holder.id().clone(),
            amount: U128(DAO_FT_TOTAL_SUPPLY * DECIMALS),
        }],
    )
    .await?;
    let staking = init_staking(&worker).await?;
    let wf_provider = init_workflow_provider(&worker).await?;
    create_dao_via_factory(
        &worker,
        &factory,
        &dao_name,
        &token_account_id,
        DAO_FT_TOTAL_SUPPLY as u32,
        24,
        staking.id(),
        wf_provider.id(),
        factory.id(),
        vec![token_holder.id()],
        0,
    )
    .await?;

    // Check on FT.
    let balance = ft_balance_of(&worker, &token_account_id, &token_holder.id())
        .await?
        .0;
    assert_eq!(
        balance,
        DAO_FT_TOTAL_SUPPLY * DECIMALS,
        "ft balance does not match"
    );

    // Storage deposit for DAO in staking.
    storage_deposit(
        &worker,
        &registrar,
        staking.id(),
        &dao_account_id,
        MIN_STORAGE_DEPOSIT,
    )
    .await?;

    // Check storage balance of DAO before register.
    storage_balance_of::<_, StorageBalance>(&worker, &staking, &dao_account_id).await?;

    // Register new dao by registrar.
    let args = json!({
        "dao_id" :  &dao_account_id,
        "vote_token_id": &token_account_id
    })
    .to_string()
    .into_bytes();
    let outcome = registrar
        .call(&worker, staking.id(), "register_new_dao")
        .args(args)
        .deposit(STANDARD_FT_STORAGE_DEPOSIT)
        .max_gas()
        .transact()
        .await?;
    assert!(outcome.is_success());
    outcome_pretty::<()>("register new dao by registrar", &outcome);

    // Check storage balance of DAO before register.
    let storage_balance_before =
        storage_balance_of::<_, StorageBalance>(&worker, &staking, &dao_account_id).await?;

    // Register token_holder in dao.
    let args = json!({
        "dao_id" : &dao_account_id,
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
    outcome_pretty::<()>("register token_holder in dao", &outcome);

    // Register delegate_1 in dao.
    let args = json!({
        "dao_id" : &dao_account_id,
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
    outcome_pretty::<()>("register delegate_1 in dao", &outcome);

    // Register delegate_2 in dao.
    let args = json!({
        "dao_id" : &dao_account_id,
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
    outcome_pretty::<()>("register delegate_2 in dao", &outcome);

    // Transfer token to staking.
    let transfer_info = format!(
        "{{\"dao_id\":\"{}\",\"delegate_id\":\"{}\"}}",
        &dao_account_id,
        token_holder.id().to_string()
    );
    ft_transfer_call(
        &worker,
        &token_holder,
        &token_account_id,
        staking.id(),
        2_000,
        None,
        transfer_info,
    )
    .await?;

    // Check transfer.
    staking_check_user(
        &worker,
        staking.id(),
        &dao_account_id,
        token_holder.id(),
        2_000,
        2_000,
        vec![(token_holder.id().to_owned(), 2_000)],
        vec![token_holder.id().to_owned()],
    )
    .await?;

    // Check on FT
    let balance = ft_balance_of(&worker, &token_account_id, &token_holder.id())
        .await?
        .0;
    assert_eq!(
        balance,
        DAO_FT_TOTAL_SUPPLY * DECIMALS - 2_000,
        "ft balance does not match"
    );

    // View token_holder weight
    check_dao_user_weight(&worker, &dao_account_id, token_holder.id(), 2_000).await?;

    let transfer_info = format!(
        "{{\"dao_id\":\"{}\",\"delegate_id\":null}}",
        &dao_account_id
    );
    ft_transfer_call(
        &worker,
        &token_holder,
        &token_account_id,
        staking.id(),
        2_000,
        None,
        transfer_info,
    )
    .await?;
    staking_check_user(
        &worker,
        staking.id(),
        &dao_account_id,
        token_holder.id(),
        4_000,
        2_000,
        vec![(token_holder.id().to_owned(), 2_000)],
        vec![token_holder.id().to_owned()],
    )
    .await?;
    check_dao_user_weight(&worker, &dao_account_id, token_holder.id(), 2_000).await?;

    // Delegate 1000 ft owned to delegate_1.
    let args = json!({
        "dao_id": &dao_account_id,
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
    outcome_pretty::<()>("delegate 1000 ft owned to delegate_1", &outcome);

    // delegate_1 check
    staking_check_user(
        &worker,
        staking.id(),
        &dao_account_id,
        delegate_1.id(),
        0,
        1_000,
        vec![],
        vec![token_holder.id().to_owned()],
    )
    .await?;
    check_dao_user_weight(&worker, &dao_account_id, delegate_1.id(), 1_000).await?;

    // Check storage balance.
    storage_balance_of::<_, StorageBalance>(&worker, &staking, &dao_account_id).await?;

    // Delegate 1000 ft owned to delegate_2.
    let args = json!({
        "dao_id": &dao_account_id,
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
    outcome_pretty::<()>("delegate 1000 ft owned to delegate_2", &outcome);

    // delegate_2 check
    staking_check_user(
        &worker,
        staking.id(),
        &dao_account_id,
        delegate_2.id(),
        0,
        1_000,
        vec![],
        vec![token_holder.id().to_owned()],
    )
    .await?;
    check_dao_user_weight(&worker, &dao_account_id, delegate_2.id(), 1_000).await?;

    storage_balance_of::<_, StorageBalance>(&worker, &staking, &dao_account_id).await?;

    // Check token_holder.
    staking_check_user(
        &worker,
        staking.id(),
        &dao_account_id,
        token_holder.id(),
        4_000,
        2_000,
        vec![
            (token_holder.id().to_owned(), 2_000),
            (delegate_1.id().to_owned(), 1000),
            (delegate_2.id().to_owned(), 1000),
        ],
        vec![token_holder.id().to_owned()],
    )
    .await?;
    check_dao_user_weight(&worker, &dao_account_id, token_holder.id(), 2_000).await?;

    // Delegate delegated by delegate_1 to delegate_2.
    let args = json!({
        "dao_id": &dao_account_id,
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
    outcome_pretty::<()>(
        "delegate delegated tokens from delegate_1 to delegate_2",
        &outcome,
    );

    // delegate_1 check
    staking_check_user(
        &worker,
        staking.id(),
        &dao_account_id,
        delegate_1.id(),
        0,
        0,
        vec![],
        vec![],
    )
    .await?;
    check_dao_user_weight(&worker, &dao_account_id, delegate_1.id(), 0).await?;

    // delegate_2 check
    staking_check_user(
        &worker,
        staking.id(),
        &dao_account_id,
        delegate_2.id(),
        0,
        2_000,
        vec![],
        vec![token_holder.id().to_owned()],
    )
    .await?;
    check_dao_user_weight(&worker, &dao_account_id, delegate_2.id(), 2_000).await?;

    // Check storage balance.
    storage_balance_of::<_, StorageBalance>(&worker, &staking, &dao_account_id).await?;

    // Check delegation delegated.
    staking_check_user(
        &worker,
        staking.id(),
        &dao_account_id,
        token_holder.id(),
        4_000,
        2_000,
        vec![
            (token_holder.id().to_owned(), 2_000),
            (delegate_2.id().to_owned(), 2_000),
        ],
        vec![token_holder.id().to_owned()],
    )
    .await?;
    check_dao_user_weight(&worker, &dao_account_id, token_holder.id(), 2_000).await?;

    // Undelegate 500 ft from delegate_2.
    let args = json!({
        "dao_id": &dao_account_id,
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
    outcome_pretty::<()>("undelegate 500 ft from delegate_2", &outcome);

    // delegate_2 check
    staking_check_user(
        &worker,
        staking.id(),
        &dao_account_id,
        delegate_2.id(),
        0,
        1_500,
        vec![],
        vec![token_holder.id().to_owned()],
    )
    .await?;
    check_dao_user_weight(&worker, &dao_account_id, delegate_2.id(), 1_500).await?;

    // Check token_holder.
    staking_check_user(
        &worker,
        staking.id(),
        &dao_account_id,
        token_holder.id(),
        4_000,
        2_000,
        vec![
            (token_holder.id().to_owned(), 2_000),
            (delegate_2.id().to_owned(), 1_500),
        ],
        vec![token_holder.id().to_owned()],
    )
    .await?;
    check_dao_user_weight(&worker, &dao_account_id, token_holder.id(), 2_000).await?;

    // Withdraw 500 ft.
    let args = json!({
        "dao_id": &dao_account_id,
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
    outcome_pretty::<()>("withdraw 500 ft", &outcome);

    // Withdraw check.
    staking_check_user(
        &worker,
        staking.id(),
        &dao_account_id,
        token_holder.id(),
        3_500,
        2_000,
        vec![
            (token_holder.id().to_owned(), 2_000),
            (delegate_2.id().to_owned(), 1_500),
        ],
        vec![token_holder.id().to_owned()],
    )
    .await?;

    // Check on FT
    let balance = ft_balance_of(&worker, &token_account_id, &token_holder.id())
        .await?
        .0;
    assert_eq!(
        balance,
        DAO_FT_TOTAL_SUPPLY * DECIMALS - 3_500,
        "ft balance does not match"
    );

    // Undelegate rest - 1500 ft.
    let args = json!({
        "dao_id": &dao_account_id,
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
    outcome_pretty::<()>("undelegate rest - 1500 ft", &outcome);

    // delegate_2 check
    staking_check_user(
        &worker,
        staking.id(),
        &dao_account_id,
        delegate_2.id(),
        0,
        0,
        vec![],
        vec![],
    )
    .await?;
    check_dao_user_weight(&worker, &dao_account_id, delegate_2.id(), 0).await?;

    // Check undelegation.
    staking_check_user(
        &worker,
        staking.id(),
        &dao_account_id,
        token_holder.id(),
        3_500,
        2_000,
        vec![(token_holder.id().to_owned(), 2_000)],
        vec![token_holder.id().to_owned()],
    )
    .await?;

    // token_holder token weight check
    check_dao_user_weight(&worker, &dao_account_id, token_holder.id(), 2_000).await?;

    // Check storage balance.
    storage_balance_of::<_, StorageBalance>(&worker, &staking, &dao_account_id).await?;

    // Undelege all from self
    let args = json!({
        "dao_id": &dao_account_id,
        "delegate_id": token_holder.id(),
        "amount": U128::from(2_000),
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
    outcome_pretty::<()>("undelegate all from self - 2000 ft", &outcome);

    // Check undelegate
    staking_check_user(
        &worker,
        staking.id(),
        &dao_account_id,
        token_holder.id(),
        3_500,
        0,
        vec![],
        vec![],
    )
    .await?;
    // token_holder token weight check
    check_dao_user_weight(&worker, &dao_account_id, token_holder.id(), 0).await?;

    // Withdraw rest - 3000 ft.
    let args = json!({
        "dao_id": &dao_account_id,
        "amount": U128::from(3_500),
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
    outcome_pretty::<()>("withdraw rest - 3500 ft", &outcome);

    // Withdraw check.
    staking_check_user(
        &worker,
        staking.id(),
        &dao_account_id,
        token_holder.id(),
        0,
        0,
        vec![],
        vec![],
    )
    .await?;
    // token_holder token weight check
    check_dao_user_weight(&worker, &dao_account_id, token_holder.id(), 0).await?;

    // Check on FT
    let balance = ft_balance_of(&worker, &token_account_id, &token_holder.id())
        .await?
        .0;
    assert_eq!(
        balance,
        DAO_FT_TOTAL_SUPPLY * DECIMALS,
        "ft balance does not match"
    );

    // Unregister token_holder in dao.
    let args = json!({
        "dao_id": &dao_account_id,
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
    outcome_pretty::<()>("unregister token_holder in dao", &outcome);

    let args = json!({
        "dao_id": &dao_account_id,
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
        "dao_id": &dao_account_id,
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
    outcome_pretty::<()>("unregister delegate_1 in dao", &outcome);

    let args = json!({
        "dao_id": &dao_account_id,
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
        "dao_id": &dao_account_id,
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
    outcome_pretty::<()>("unregister delegate_2 in dao", &outcome);

    let args = json!({
        "dao_id": &dao_account_id,
        "account_id": delegate_2.id(),
    })
    .to_string()
    .into_bytes();
    assert!(staking
        .view(&worker, VIEW_METHOD_GET_USER, args)
        .await
        .is_err());

    // Check storage balance after unregistering all 3 users.
    let storage_balance_after =
        storage_balance_of::<_, StorageBalance>(&worker, &staking, &dao_account_id).await?;
    assert_eq!(
        storage_balance_before, storage_balance_after,
        "{}",
        "Storage balance does not equal."
    );

    // Storage unregister DAO.
    let args = json!({}).to_string().into_bytes();
    let outcome = worker
        .call(
            &staking,
            "storage_unregister",
            args,
            Some(MAX_GAS),
            Some(ONE_YOCTO),
        )
        .await?;
    assert!(outcome.is_success());
    let args = json!({
        "dao_id": &dao_account_id,
    })
    .to_string()
    .into_bytes();
    assert!(staking
        .view(&worker, VIEW_METHOD_FT_TOTAL_SUPPLY, args)
        .await
        .is_ok());
    Ok(())
}

#[tokio::test]
/// Withdraw amount (1500) > vote_amount (3000) - delegated amount (2000).
async fn staking_withdraw_invalid_amount() -> anyhow::Result<()> {
    let dao_name = "test_dao";
    let ft_name = "dao_token";
    let worker = workspaces::sandbox().await?;
    let registrar = worker.dev_create_account().await?;
    let token_holder = worker.dev_create_account().await?;
    let delegate_1 = worker.dev_create_account().await?;
    let delegate_2 = worker.dev_create_account().await?;
    let ft_factory = init_ft_factory(&worker).await?;
    let factory = init_dao_factory(&worker).await?;
    let dao_account_id = AccountId::try_from(format!("{}.{}", dao_name, factory.id()))
        .expect("invalid dao account id");
    let token_account_id = AccountId::try_from(format!("{}.{}", ft_name, ft_factory.id()))
        .expect("invalid ft account id");
    create_ft_via_factory(
        &worker,
        &ft_factory,
        ft_name,
        dao_account_id.as_str(),
        DAO_FT_TOTAL_SUPPLY,
        24,
        None,
        None,
        vec![InitDistribution {
            account_id: token_holder.id().clone(),
            amount: U128(DAO_FT_TOTAL_SUPPLY * DECIMALS),
        }],
    )
    .await?;
    let staking = init_staking(&worker).await?;
    let wf_provider = init_workflow_provider(&worker).await?;
    create_dao_via_factory(
        &worker,
        &factory,
        &dao_name,
        &token_account_id,
        DAO_FT_TOTAL_SUPPLY as u32,
        24,
        staking.id(),
        wf_provider.id(),
        factory.id(),
        vec![token_holder.id()],
        0,
    )
    .await?;

    // Check on FT.
    let balance = ft_balance_of(&worker, &token_account_id, &token_holder.id())
        .await?
        .0;
    assert_eq!(
        balance,
        DAO_FT_TOTAL_SUPPLY * DECIMALS,
        "ft balance does not match"
    );

    // Storage deposit for DAO in staking.
    storage_deposit(
        &worker,
        &registrar,
        staking.id(),
        &dao_account_id,
        MIN_STORAGE_DEPOSIT,
    )
    .await?;

    // Check storage balance of DAO before register.
    storage_balance_of::<_, StorageBalance>(&worker, &staking, &dao_account_id).await?;

    // Register new dao by registrar.
    let args = json!({
        "dao_id" :  &dao_account_id,
        "vote_token_id": &token_account_id
    })
    .to_string()
    .into_bytes();
    let outcome = registrar
        .call(&worker, staking.id(), "register_new_dao")
        .args(args)
        .max_gas()
        .deposit(STANDARD_FT_STORAGE_DEPOSIT)
        .transact()
        .await?;
    assert!(outcome.is_success());
    outcome_pretty::<()>("register new dao by registrar", &outcome);

    // Storage deposit staking in fungible_token.
    storage_deposit(
        &worker,
        &registrar,
        &token_account_id,
        staking.id(),
        MIN_STORAGE_DEPOSIT,
    )
    .await?;

    // Register token_holder in dao.
    let args = json!({
        "dao_id" : &dao_account_id,
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
    outcome_pretty::<()>("register token_holder in dao", &outcome);

    // Register delegate_1 in dao.
    let args = json!({
        "dao_id" : &dao_account_id,
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
    outcome_pretty::<()>("register delegate_1 in dao", &outcome);

    // Register delegate_2 in dao.
    let args = json!({
        "dao_id" : &dao_account_id,
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
    outcome_pretty::<()>("register delegate_2 in dao", &outcome);

    // Transfer token to staking.
    let transfer_info = format!(
        "{{\"dao_id\":\"{}\",\"delegate_id\":null}}",
        &dao_account_id
    );
    ft_transfer_call(
        &worker,
        &token_holder,
        &token_account_id,
        staking.id(),
        2_000 + 1_000,
        None,
        transfer_info,
    )
    .await?;

    // Check transfer.
    staking_check_user(
        &worker,
        staking.id(),
        &dao_account_id,
        token_holder.id(),
        3_000,
        0,
        vec![],
        vec![],
    )
    .await?;

    // Check on FT
    let balance = ft_balance_of(&worker, &token_account_id, &token_holder.id())
        .await?
        .0;
    assert_eq!(
        balance,
        DAO_FT_TOTAL_SUPPLY * DECIMALS - 3_000,
        "ft balance does not match"
    );

    // View token_holder weight
    check_dao_user_weight(&worker, &dao_account_id, token_holder.id(), 0).await?;

    // Delegate 1000 ft owned to delegate_1.
    let args = json!({
        "dao_id": &dao_account_id,
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
    outcome_pretty::<()>("delegate 1000 ft owned to delegate_1", &outcome);

    // Delegate 1000 ft owned to delegate_2.
    let args = json!({
        "dao_id": &dao_account_id,
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
    // View delegate_2 weight
    check_dao_user_weight(&worker, &dao_account_id, delegate_2.id(), 1_000).await?;

    // Withdraw 1500 when 3000 is deposited and 2000 is delegated.
    let args = json!({
        "dao_id": &dao_account_id,
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

    // View weights
    check_dao_user_weight(&worker, &dao_account_id, token_holder.id(), 0).await?;
    check_dao_user_weight(&worker, &dao_account_id, delegate_1.id(), 1_000).await?;
    check_dao_user_weight(&worker, &dao_account_id, delegate_2.id(), 1_000).await?;
    Ok(())
}
