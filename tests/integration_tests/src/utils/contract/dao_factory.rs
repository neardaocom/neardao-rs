use near_sdk::{json_types::Base64VecU8, ONE_NEAR};
use serde::{Deserialize, Serialize};
use serde_json::json;
use workspaces::Account;
use workspaces::{network::DevAccountDeployer, AccountId, Contract, DevNetwork, Worker};

use crate::utils::contract::dao::dao_init_args;
use crate::utils::{
    get_dao_factory_wasm, get_factory_v1, get_factory_v2, get_factory_v2_migration, outcome_pretty,
};

pub async fn init_dao_factory<T>(worker: &Worker<T>) -> anyhow::Result<Contract>
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

pub async fn create_dao_via_factory<T>(
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
    init_distribution: u32,
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
        init_distribution,
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

pub async fn deploy_upgrade_dao_factory<T>(
    worker: &Worker<T>,
    factory: Option<&Account>,
    version: &str,
) -> anyhow::Result<Contract>
where
    T: DevNetwork,
{
    let (wasm_blob, migration_type) = match version {
        "v1" => (get_factory_v1(), "only_migration"),
        "v2_migration" => (get_factory_v2_migration(), "new_migration_bin"),
        "v2" => (get_factory_v2(), "new_upgrade_bin"),
        _ => panic!("Invalid upgrade dao factory version"),
    };

    let factory_contract = if version == "v1" {
        let factory_contract = worker.dev_deploy(&std::fs::read(wasm_blob)?).await?;
        let tags: Vec<String> = vec![];
        let args = json!({
            "tags" :tags,
        })
        .to_string()
        .into_bytes();
        let outcome = factory_contract
            .call(&worker, "new")
            .args(args)
            .max_gas()
            .transact()
            .await?;
        outcome_pretty::<()>("dao factory init", &outcome);
        assert!(outcome.is_success(), "dao factory init failed");
        factory_contract
    } else {
        let factory = factory.expect("invalid use - expected factory contract");
        let factory_contract = factory.deploy(worker, &std::fs::read(wasm_blob)?).await?;
        let args = json!({
            "type": migration_type,
        })
        .to_string()
        .into_bytes();
        let outcome = factory
            .call(&worker, factory.id(), "migrate")
            .args(args)
            .max_gas()
            .transact()
            .await?;
        outcome_pretty::<()>("dao factory upgrade", &outcome);
        assert!(outcome.is_success(), "dao factory upgrade failed");
        factory_contract.into_result()?
    };
    Ok(factory_contract)
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
