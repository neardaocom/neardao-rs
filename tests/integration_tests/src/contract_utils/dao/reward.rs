use data::workflow::integration::skyward::{Skyward1, Skyward1ProposeOptions};
use library::workflow::settings::{ProposeSettings, TemplateSettings};
use near_sdk::json_types::U128;
use serde_json::json;
use workspaces::{network::Sandbox, Account, AccountId, Contract, DevNetwork, Worker};

use crate::{
    contract_utils::dao::{
        types::proposal::{ProposalCreateInput, ProposalState},
        view::{proposal, votes},
    },
    utils::outcome_pretty,
};

use super::types::reward::Asset;

pub(crate) async fn withdraw_rewards<T>(
    worker: &Worker<T>,
    caller: &Account,
    dao: &AccountId,
    rewards: Vec<u16>,
    asset: Asset,
) -> anyhow::Result<()>
where
    T: DevNetwork,
{
    let args = json!({
        "reward_ids": rewards,
        "asset": asset
    })
    .to_string()
    .into_bytes();
    let outcome = caller
        .call(&worker, dao, "withdraw_rewards")
        .args(args)
        .max_gas()
        .transact()
        .await?;
    outcome_pretty::<U128>("dao withdraw_rewards", &outcome);
    assert!(outcome.is_success(), "dao withdraw_rewards");

    Ok(())
}
