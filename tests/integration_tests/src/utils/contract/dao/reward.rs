use near_sdk::json_types::U128;
use serde_json::json;
use workspaces::{Account, AccountId, DevNetwork, Worker};

use crate::{types::AssetId, utils::outcome_pretty};

pub async fn withdraw_rewards<T>(
    worker: &Worker<T>,
    caller: &Account,
    dao: &AccountId,
    rewards: Vec<u16>,
    asset_id: AssetId,
) -> anyhow::Result<()>
where
    T: DevNetwork,
{
    let args = json!({
        "reward_ids": rewards,
        "asset_id": asset_id
    })
    .to_string()
    .into_bytes();
    let outcome = caller
        .call(&worker, dao, "claim_rewards")
        .args(args)
        .max_gas()
        .transact()
        .await?;
    outcome_pretty::<U128>("dao claim_rewards", &outcome);
    assert!(outcome.is_success(), "dao claim_rewards");
    Ok(())
}
