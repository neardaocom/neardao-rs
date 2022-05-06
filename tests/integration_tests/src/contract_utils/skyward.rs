//! Helper methods to interact with skyward contract.
//! Reference: https://github.com/skyward-finance/contracts/tree/master/skyward

use near_sdk::{
    json_types::{U128, U64},
    BlockHeight, ONE_NEAR,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use workspaces::{network::DevAccountDeployer, AccountId, Contract, DevNetwork, Worker};

use crate::utils::{
    get_skyward_wasm, outcome_pretty, parse_view_result, view_outcome_pretty, TimestampSec,
    WrappedBalance,
};

pub type ViewSales = Vec<SaleOutput>;

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
    outcome_pretty::<()>("skyward init", &outcome);
    assert!(outcome.is_success(), "skyward init failed");
    Ok(skyward)
}

fn default_vesting_schedule() -> VestingIntervalInput {
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

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct SaleOutput {
    pub sale_id: u64,
    pub title: String,
    pub url: Option<String>,
    pub permissions_contract_id: Option<AccountId>,

    pub owner_id: AccountId,

    pub out_tokens: Vec<SaleOutputOutToken>,

    pub in_token_account_id: AccountId,
    pub in_token_remaining: WrappedBalance,
    pub in_token_paid_unclaimed: WrappedBalance,
    pub in_token_paid: WrappedBalance,

    pub total_shares: WrappedBalance,

    pub start_time: U128,
    pub duration: U64,
    pub remaining_duration: U64,

    pub subscription: Option<SubscriptionOutput>,

    pub current_time: U64,
    pub current_block_height: BlockHeight,
    pub start_block_height: BlockHeight,
    pub end_block_height: Option<BlockHeight>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct SaleOutputOutToken {
    pub token_account_id: AccountId,
    pub remaining: WrappedBalance,
    pub distributed: WrappedBalance,
    pub treasury_unclaimed: Option<WrappedBalance>,
    pub referral_bpt: Option<u16>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct SubscriptionOutput {
    pub remaining_in_balance: WrappedBalance,
    pub spent_in_balance: WrappedBalance,
    pub unclaimed_out_balances: Vec<WrappedBalance>,
    pub claimed_out_balance: Vec<WrappedBalance>,
    pub shares: WrappedBalance,
    pub referral_id: Option<AccountId>,
}

pub(crate) async fn skyward_sales<T>(
    worker: &Worker<T>,
    skyward: &Contract,
) -> anyhow::Result<ViewSales>
where
    T: DevNetwork,
{
    let args = json!({
        "account_id": null,
        "from_index": null,
        "limit": null
    })
    .to_string()
    .into_bytes();
    let outcome = skyward.view(&worker, "get_sales", args).await?;
    view_outcome_pretty::<ViewSales>("skyward view sales", &outcome);
    let sales = parse_view_result::<ViewSales>(&outcome).expect("failed to parse skyward sales");
    Ok(sales)
}

pub(crate) async fn check_sale<T>(
    worker: &Worker<T>,
    skyward: &Contract,
    expected_sale_id: u64,
    expected_title: String,
    expected_url: String,
    expected_token: &AccountId,
    expected_token_amount: u128,
    expected_receiver_token_id: &AccountId,
) -> anyhow::Result<()>
where
    T: DevNetwork,
{
    let sales = skyward_sales(worker, skyward).await?;
    let sale = sales
        .into_iter()
        .find(|s| s.sale_id == expected_sale_id)
        .expect("skyward sale with expected id not found");
    assert_eq!(
        sale.title, expected_title,
        "skyward sale title does not match"
    );
    assert_eq!(
        sale.url,
        Some(expected_url),
        "skyward sale url does not match"
    );
    assert_eq!(
        sale.in_token_account_id,
        expected_receiver_token_id.to_owned(),
        "skyward sale expected input token does not match"
    );
    assert_eq!(
        sale.out_tokens[0].token_account_id,
        expected_token.to_owned(),
        "skyward sale title does not match"
    );
    assert_eq!(
        sale.out_tokens[0].remaining,
        expected_token_amount.into(),
        "skyward sale title does not match"
    );
    Ok(())
}
