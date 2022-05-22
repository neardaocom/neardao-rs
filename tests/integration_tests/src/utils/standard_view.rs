use near_sdk::json_types::U128;
use serde_json::json;
use workspaces::{AccountId, Contract, DevNetwork, Worker};

use super::{parse_view_result, view_outcome_pretty};

pub async fn ft_balance_of<T>(
    worker: &Worker<T>,
    ft_contract: &AccountId,
    account_id: &AccountId,
) -> anyhow::Result<U128>
where
    T: DevNetwork,
{
    let args = json!({ "account_id": account_id.to_string() })
        .to_string()
        .into_bytes();
    let outcome = worker.view(&ft_contract, "ft_balance_of", args).await?;
    let title = format!(
        "view ft_balance_of account: {} on contract: {}",
        account_id.as_str(),
        ft_contract.as_str(),
    );
    view_outcome_pretty::<U128>(&title, &outcome);
    let amount = parse_view_result::<U128>(&outcome).expect("failed to parse ft_balance_of amount");
    Ok(amount)
}

pub async fn storage_balance_of<T, U>(
    worker: &Worker<T>,
    contract: &Contract,
    account_id: &AccountId,
) -> anyhow::Result<U>
where
    T: DevNetwork,
    U: for<'de> serde::Deserialize<'de> + std::fmt::Debug,
{
    let args = json!({ "account_id": account_id.to_string() })
        .to_string()
        .into_bytes();
    let outcome = contract.view(&worker, "storage_balance_of", args).await?;
    let title = format!(
        "view storage_balance_of account: {} on contract: {}",
        account_id.as_str(),
        contract.id().as_str(),
    );
    view_outcome_pretty::<U>(&title, &outcome);
    let amount =
        parse_view_result::<U>(&outcome).expect("failed to parse storage_balance_of amount");
    Ok(amount)
}

pub async fn storage_minimum_balance<T>(
    worker: &Worker<T>,
    contract: &Contract,
) -> anyhow::Result<U128>
where
    T: DevNetwork,
{
    let args = json!({}).to_string().into_bytes();
    let outcome = contract
        .view(&worker, "storage_minimum_balance", args)
        .await?;
    let title = format!(
        "view storage_minimum_balance on contract: {}",
        contract.id().as_str(),
    );
    view_outcome_pretty::<U128>(&title, &outcome);
    let amount = parse_view_result::<U128>(&outcome)
        .expect("failed to parse storage_minimum_balance amount");
    Ok(amount)
}
