use near_sdk::json_types::U128;
use serde_json::json;
use workspaces::{Account, AccountId, DevNetwork, Worker};

use crate::{types::StorageBalance, utils::outcome_pretty};

pub async fn storage_deposit<T>(
    worker: &Worker<T>,
    caller: &Account,
    contract: &AccountId,
    account_id: &AccountId,
    deposit: u128,
) -> anyhow::Result<()>
where
    T: DevNetwork,
{
    let args = json!({ "account_id": account_id }).to_string().into_bytes();
    let outcome = caller
        .call(&worker, contract, "storage_deposit")
        .args(args)
        .max_gas()
        .deposit(deposit)
        .transact()
        .await?;
    assert!(outcome.is_success());
    let msg = format!(
        "{} calls storage deposit in {} for account {}",
        caller.id(),
        contract.as_str(),
        account_id
    );
    outcome_pretty::<StorageBalance>(&msg, &outcome);
    Ok(())
}

pub async fn ft_transfer_call<T>(
    worker: &Worker<T>,
    caller: &Account,
    contract: &AccountId,
    receiver_id: &AccountId,
    amount: u128,
    memo: Option<String>,
    msg: String,
) -> anyhow::Result<()>
where
    T: DevNetwork,
{
    let args = json!({
        "receiver_id": receiver_id,
        "amount": U128(amount),
        "memo": memo,
        "msg": msg
    })
    .to_string()
    .into_bytes();
    let outcome = caller
        .call(&worker, contract, "ft_transfer_call")
        .args(args)
        .max_gas()
        .deposit(1)
        .transact()
        .await?;
    assert!(outcome.is_success());
    let msg = format!(
        "{} calls ft_transfer_call in {} for account {}",
        caller.id(),
        contract,
        receiver_id
    );
    outcome_pretty::<StorageBalance>(&msg, &outcome);
    Ok(())
}

pub async fn ft_transfer<T>(
    worker: &Worker<T>,
    caller: &Account,
    contract: &AccountId,
    receiver_id: &AccountId,
    amount: u128,
) -> anyhow::Result<()>
where
    T: DevNetwork,
{
    let args = format!(
        "{{\"receiver_id\":\"{}\",\"amount\":\"{}\",\"memo\":null}}",
        receiver_id, amount
    )
    .into_bytes();
    let outcome = caller
        .call(&worker, contract, "ft_transfer")
        .args(args)
        .max_gas()
        .deposit(1)
        .transact()
        .await?;
    assert!(outcome.is_success());
    let _msg = format!(
        "{} calls ft_transfer in {} for account {}",
        caller.id(),
        contract,
        receiver_id
    );
    Ok(())
}
