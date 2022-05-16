use near_sdk::{json_types::Base64VecU8, ONE_NEAR};
use serde::{Deserialize, Serialize};
use serde_json::json;
use workspaces::{network::DevAccountDeployer, AccountId, Contract, DevNetwork, Worker};

use crate::{
    contract_utils::dao::init::dao_init_args,
    utils::{get_dao_factory_wasm, outcome_pretty},
};

pub(crate) async fn init_dao_factory<T>(worker: &Worker<T>) -> anyhow::Result<Contract>
where
    T: DevNetwork,
{
    let factory_blob_path = get_dao_factory_wasm();
    let factory = worker
        .dev_deploy(&std::fs::read(factory_blob_path)?)
        .await?;
    let tags: Vec<String> = vec![];
    let args = json!({
        "tags" :tags,
    })
    .to_string()
    .into_bytes();
    let outcome = factory
        .call(&worker, "new")
        .args(args)
        .max_gas()
        .transact()
        .await?;
    outcome_pretty::<()>("dao factory init", &outcome);
    assert!(outcome.is_success(), "dao factory init failed");
    Ok(factory)
}

pub(crate) async fn create_dao_via_factory<T>(
    worker: &Worker<T>,
    factory: &Contract,
    dao_name: &str,
    token_id: &AccountId,
    total_supply: u32,
    decimals: u8,
    staking_id: &AccountId,
    provider_id: &AccountId,
    admin_id: &AccountId,
    council_members: Vec<&AccountId>,
) -> anyhow::Result<()>
where
    T: DevNetwork,
{
    let dao_new_input = dao_init_args(
        token_id.clone(),
        total_supply,
        decimals,
        staking_id.clone(),
        provider_id.clone(),
        admin_id.clone(),
        council_members,
    )
    .0;
    let json_input =
        serde_json::to_string(&dao_new_input).expect("failed to serialize dao new input");
    let args: Base64VecU8 = Base64VecU8(json_input.into_bytes());
    let dao_info = DaoInfo {
        founded_s: 0,
        name: dao_name.into(),
        description: "test".into(),
        ft_name: token_id.to_string(),
        ft_amount: total_supply.into(),
        tags: vec![],
    };
    let args = json!({
        "name": dao_name,
        "info": dao_info,
        "args": args
    })
    .to_string()
    .into_bytes();
    let outcome = factory
        .call(&worker, "create")
        .args(args)
        .max_gas()
        .deposit(50 * ONE_NEAR)
        .transact()
        .await?;
    outcome_pretty::<bool>("create dao via dao factory init", &outcome);
    assert!(
        outcome.is_success(),
        "create dao via dao factory init failed"
    );
    Ok(())
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct DaoInfo {
    pub founded_s: u64,
    pub name: String,
    pub description: String,
    pub ft_name: String,
    pub ft_amount: u32,
    pub tags: Vec<u8>,
}
