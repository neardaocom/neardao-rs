use near_sdk::{
    json_types::{Base64VecU8, U128},
    ONE_NEAR,
};
use serde_json::json;
use workspaces::{network::DevAccountDeployer, Contract, DevNetwork, Worker};

use crate::{
    types::{default_ft_metadata, FtSettings, FungibleTokenMetadata, InitDistribution},
    utils::{get_ft_factory_wasm, outcome_pretty},
};

pub async fn init_ft_factory<T>(worker: &Worker<T>) -> anyhow::Result<Contract>
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

pub async fn create_ft_via_factory<T>(
    worker: &Worker<T>,
    ft_factory: &Contract,
    ft_name: &str,
    owner_id: &str,
    total_supply: u128,
    decimals: u32,
    metadata: Option<FungibleTokenMetadata>,
    settings: Option<FtSettings>,
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
