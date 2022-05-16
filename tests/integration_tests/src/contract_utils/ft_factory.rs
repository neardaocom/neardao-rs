use near_sdk::{
    json_types::{Base64VecU8, U128},
    ONE_NEAR,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use workspaces::{network::DevAccountDeployer, AccountId, Contract, DevNetwork, Worker};

use crate::utils::{get_ft_factory_wasm, outcome_pretty};

pub(crate) async fn init_ft_factory<T>(worker: &Worker<T>) -> anyhow::Result<Contract>
where
    T: DevNetwork,
{
    let factory_blob_path = get_ft_factory_wasm();
    let factory = worker
        .dev_deploy(&std::fs::read(factory_blob_path)?)
        .await?;
    let args = json!({}).to_string().into_bytes();
    let outcome = factory
        .call(&worker, "new")
        .args(args)
        .max_gas()
        .transact()
        .await?;
    outcome_pretty::<()>("ft factory init", &outcome);
    assert!(outcome.is_success(), "ft factory init failed");
    Ok(factory)
}

pub(crate) async fn create_ft_via_factory<T>(
    worker: &Worker<T>,
    ft_factory: &Contract,
    ft_name: &str,
    owner_id: &str,
    total_supply: u128,
    decimals: u32,
    metadata: Option<FungibleTokenMetadata>,
    settings: Option<Settings>,
    init_distribution: Vec<InitDistribution>,
) -> anyhow::Result<()>
where
    T: DevNetwork,
{
    let ft_args = json!({
        "owner_id": owner_id,
        "total_supply": U128(total_supply * 10u128.pow(decimals)),
        "metadata": metadata.unwrap_or(default_ft_metadata()),
        "settings": settings,
        "init_distribution": init_distribution,
    })
    .to_string()
    .into_bytes();
    let base64_ft_args = Base64VecU8(ft_args);
    let args = json!({
        "name": ft_name,
        "args": base64_ft_args,
    })
    .to_string()
    .into_bytes();
    let outcome = ft_factory
        .call(&worker, "create")
        .args(args)
        .max_gas()
        .deposit(3 * ONE_NEAR)
        .transact()
        .await?;
    outcome_pretty::<bool>("create ft via ft factory init", &outcome);
    assert!(outcome.is_success(), "create ft via ft factory init failed");
    Ok(())
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct FungibleTokenMetadata {
    pub spec: String,
    pub name: String,
    pub symbol: String,
    pub icon: Option<String>,
    pub reference: Option<String>,
    pub reference_hash: Option<Base64VecU8>,
    pub decimals: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Settings {
    /// Account allowed to change these settings.
    owner_id: AccountId,
    mint_allowed: bool,
    burn_allowed: bool,
    /// Account of contract allowed to provide new version.
    /// If not set then upgrade is not allowed.
    /// TODO: Implement.
    upgrade_provider: Option<AccountId>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct InitDistribution {
    pub account_id: AccountId,
    pub amount: U128,
}

const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 288 288'%3E%3Cg id='l' data-name='l'%3E%3Cpath d='M187.58,79.81l-30.1,44.69a3.2,3.2,0,0,0,4.75,4.2L191.86,103a1.2,1.2,0,0,1,2,.91v80.46a1.2,1.2,0,0,1-2.12.77L102.18,77.93A15.35,15.35,0,0,0,90.47,72.5H87.34A15.34,15.34,0,0,0,72,87.84V201.16A15.34,15.34,0,0,0,87.34,216.5h0a15.35,15.35,0,0,0,13.08-7.31l30.1-44.69a3.2,3.2,0,0,0-4.75-4.2L96.14,186a1.2,1.2,0,0,1-2-.91V104.61a1.2,1.2,0,0,1,2.12-.77l89.55,107.23a15.35,15.35,0,0,0,11.71,5.43h3.13A15.34,15.34,0,0,0,216,201.16V87.84A15.34,15.34,0,0,0,200.66,72.5h0A15.35,15.35,0,0,0,187.58,79.81Z'/%3E%3C/g%3E%3C/svg%3E";
pub const FT_METADATA_SPEC: &str = "ft-1.0.0";

fn default_ft_metadata() -> FungibleTokenMetadata {
    FungibleTokenMetadata {
        spec: FT_METADATA_SPEC.to_string(),
        name: "Example NEAR fungible token".to_string(),
        symbol: "EXAMPLE".to_string(),
        icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
        reference: None,
        reference_hash: None,
        decimals: 24,
    }
}
