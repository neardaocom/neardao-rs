use near_sdk::json_types::U128;
use serde_json::json;
use workspaces::{network::DevAccountDeployer, AccountId, Contract, DevNetwork, Worker};

use crate::{
    types::User,
    utils::{get_staking_wasm, outcome_pretty, parse_view_result, view_outcome_pretty},
};

pub async fn init_staking<T>(worker: &Worker<T>) -> anyhow::Result<Contract>
where
    T: DevNetwork,
{
    let staking_blob_path = get_staking_wasm();
    let staking = worker
        .dev_deploy(&std::fs::read(staking_blob_path)?)
        .await?;
    let args = json!({}).to_string().into_bytes();
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

pub async fn dao_user_weight<T>(
    worker: &Worker<T>,
    dao: &AccountId,
    user: &AccountId,
) -> anyhow::Result<Option<U128>>
where
    T: DevNetwork,
{
    let args = json!({
        "account_id": user,
    })
    .to_string()
    .into_bytes();
    let outcome = worker.view(&dao, "user_vote_weight", args).await?;
    let msg = format!("view {} token weight", user);
    view_outcome_pretty::<Option<U128>>(&msg, &outcome);
    let weight = parse_view_result::<Option<U128>>(&outcome).flatten();
    Ok(weight)
}

pub async fn check_dao_user_weight<T>(
    worker: &Worker<T>,
    dao: &AccountId,
    user: &AccountId,
    expected_weight: u128,
) -> anyhow::Result<()>
where
    T: DevNetwork,
{
    let user_weight = dao_user_weight(worker, dao, user).await?;
    assert_eq!(
        user_weight.unwrap_or(0.into()).0,
        expected_weight,
        "expected dao user weight does not match actual"
    );
    Ok(())
}

pub async fn staking_dao_user<T>(
    worker: &Worker<T>,
    staking: &AccountId,
    dao: &AccountId,
    user: &AccountId,
) -> anyhow::Result<Option<User>>
where
    T: DevNetwork,
{
    let args = json!({
        "dao_id": dao,
        "account_id": user,
    })
    .to_string()
    .into_bytes();
    let outcome = worker.view(&staking, "dao_get_user", args).await?;
    view_outcome_pretty::<Option<User>>("staking: dao_get_user", &outcome);
    let user = parse_view_result::<Option<User>>(&outcome).flatten();
    Ok(user)
}

pub async fn staking_check_user<T>(
    worker: &Worker<T>,
    staking: &AccountId,
    dao: &AccountId,
    user: &AccountId,
    vote_amount: u128,
    delegated_vote_amount: u128,
    delegated_amounts: Vec<(AccountId, u128)>,
    delegators: Vec<AccountId>,
) -> anyhow::Result<()>
where
    T: DevNetwork,
{
    let user_type = staking_dao_user(worker, staking, dao, user)
        .await?
        .expect("staking user not found");
    assert_eq!(
        user_type.vote_amount,
        vote_amount,
        "{}, vote amount diffs",
        user.as_str()
    );
    assert_eq!(
        user_type.delegated_vote_amount,
        delegated_vote_amount,
        "{}, delegated vote amount diffs",
        user.as_str()
    );
    assert_eq!(
        user_type.delegated_amounts,
        delegated_amounts,
        "{}, delegated amounts diffs",
        user.as_str()
    );
    assert_eq!(
        user_type.delegators,
        delegators,
        "{}, delegators diffs",
        user.as_str()
    );
    Ok(())
}
