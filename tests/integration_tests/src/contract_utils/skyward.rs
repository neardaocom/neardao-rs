use near_sdk::{json_types::U128, ONE_NEAR};
use serde::{Deserialize, Serialize};
use workspaces::{network::DevAccountDeployer, AccountId, Contract, DevNetwork, Worker};

use crate::utils::{get_skyward_wasm, outcome_pretty, TimestampSec, WrappedBalance};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct VestingIntervalInput {
    pub start_timestamp: TimestampSec,
    pub end_timestamp: TimestampSec,
    pub amount: WrappedBalance,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub(crate) struct SkywardInit {
    skyward_token_id: AccountId,
    skyward_vesting_schedule: Vec<VestingIntervalInput>,
    listing_fee_near: WrappedBalance,
    w_near_token_id: AccountId,
}

pub(crate) async fn init_skyward<T>(
    worker: &Worker<T>,
    w_near_contract: &Contract,
    settings: Option<SkywardInit>,
) -> anyhow::Result<Contract>
where
    T: DevNetwork,
{
    let skyward_blob_path = get_skyward_wasm();
    let skyward = worker
        .dev_deploy(&std::fs::read(skyward_blob_path)?)
        .await?;
    let args = if let Some(value) = settings {
        serde_json::to_string(&value).expect("Failed to serialize provided skyward init args")
    } else {
        serde_json::to_string(&default_init_settings(w_near_contract.id()))
            .expect("Failed to serialize default skyward init args")
    }
    .as_bytes()
    .to_vec();
    let outcome = skyward
        .call(&worker, "new")
        .args(args)
        .max_gas()
        .transact()
        .await?;
    outcome_pretty("skyward init", &outcome);
    assert!(outcome.is_success(), "skyward init failed");
    Ok(skyward)
}

fn defailt_vesting_schedule() -> VestingIntervalInput {
    VestingIntervalInput {
        start_timestamp: 0,
        end_timestamp: 3600,
        amount: ONE_NEAR.into(),
    }
}

fn default_init_settings(w_near_token_id: &AccountId) -> SkywardInit {
    SkywardInit {
        skyward_token_id: AccountId::try_from("skyward.token.testnet".to_string()).unwrap(),
        skyward_vesting_schedule: vec![],
        listing_fee_near: U128(ONE_NEAR),
        w_near_token_id: w_near_token_id.clone(),
    }
}
