use near_sdk::json_types::U128;
use serde_json::json;
use workspaces::{Account, AccountId, Contract, DevNetwork, Worker};

use crate::{contract_utils::dao::types::view::StorageBalance, utils::outcome_pretty};

use super::dao::types::activity::ReceiverMessage;

pub(crate) async fn storage_deposit<T>(
    worker: &Worker<T>,
    caller: &Account,
    contract: &Contract,
    account_id: &AccountId,
    deposit: u128,
) -> anyhow::Result<()>
where
    T: DevNetwork,
{
    let args = json!({ "account_id": account_id }).to_string().into_bytes();
    let outcome = caller
        .call(&worker, contract.id(), "storage_deposit")
        .args(args)
        .max_gas()
        .deposit(deposit)
        .transact()
        .await?;
    assert!(outcome.is_success());
    let msg = format!(
        "{} calls storage deposit in {} for account {}",
        caller.id(),
        contract.id(),
        account_id
    );
    outcome_pretty::<StorageBalance>(&msg, &outcome);
    Ok(())
}

pub(crate) async fn ft_transfer_call<T>(
    worker: &Worker<T>,
    caller: &Account,
    contract: &Contract,
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
        .call(&worker, contract.id(), "ft_transfer_call")
        .args(args)
        .max_gas()
        .deposit(1)
        .transact()
        .await?;
    assert!(outcome.is_success());
    let msg = format!(
        "{} calls ft_transfer_call in {} for account {}",
        caller.id(),
        contract.id(),
        receiver_id
    );
    outcome_pretty::<StorageBalance>(&msg, &outcome);
    Ok(())
}

pub(crate) async fn ft_transfer<T>(
    worker: &Worker<T>,
    caller: &Account,
    contract: &Contract,
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
        .call(&worker, contract.id(), "ft_transfer")
        .args(args)
        .max_gas()
        .deposit(1)
        .transact()
        .await?;
    assert!(outcome.is_success());
    let msg = format!(
        "{} calls ft_transfer in {} for account {}",
        caller.id(),
        contract.id(),
        receiver_id
    );
    Ok(())
}

pub(crate) fn serialized_dao_ft_receiver_msg(proposal_id: u32) -> String {
    let msg = serde_json::to_string(&ReceiverMessage { proposal_id })
        .expect("failed to serialize dao receiver msg");
    println!("msg {}", &msg);
    msg
}
