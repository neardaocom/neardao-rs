use near_sdk::{json_types::U128, ONE_NEAR};
use workspaces::network::DevAccountDeployer;

use crate::contract_utils::{
    dao::{deploy_dao, init_dao},
    functions::storage_deposit,
    fungible_token::init_fungible_token,
    skyward::init_skyward,
    staking::init_staking,
    wnear::init_wnear,
    workflow_provider::init_workflow_provider,
};

fn dao_init_args_as_bytes() -> Vec<u8> {
    vec![]
}

const DAO_FT_TOTAL_SUPPLY: u128 = 1_000_000_000;

/// Test sale create on skyward scenario as DAO with production binaries.
/// TODO: Involve factory account in the process.
/// TODO: Refactor boilerplate steps into functions.
#[tokio::test]
#[ignore = "Not finished yet."]
async fn workflow_skyward_scenario() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let member = worker.dev_create_account().await?;
    let registrar = worker.dev_create_account().await?;
    let factory = worker.dev_create_account().await?;

    // Contracts init.
    let wnear = init_wnear(&worker).await?;
    let skyward = init_skyward(&worker, &wnear, None).await?;
    let staking = init_staking(&worker, registrar.id()).await?;
    let wf_provider = init_workflow_provider(&worker).await?;
    let dao = deploy_dao(&worker, &factory).await?;
    let token = init_fungible_token(&worker, dao.id(), DAO_FT_TOTAL_SUPPLY).await?;

    init_dao(
        &worker,
        &factory,
        &dao,
        token.id(),
        DAO_FT_TOTAL_SUPPLY as u32,
        staking.id(),
        wf_provider.id(),
        factory.id(),
        vec![member.id()],
    )
    .await?;

    // Storage deposit staking in fungible_token.
    let _ = storage_deposit(&worker, &factory, &token, staking.id()).await?;

    /*     let args = dao_init_args_as_bytes();
    // DAO init. Dao should register init members in FT and immediatelly send them their tokens.
    let outcome = factory
        .call(&worker, dao.id(), "new")
        .args(args)
        .max_gas()
        .deposit(MIN_REGISTER_DEPOSIT)
        .transact()
        .await?;
    assert!(outcome.is_success());
    outcome_pretty("init dao", &outcome);

    // Register member in dao.
    let args = json!({
        "dao_id" : dao.id(),
    })
    .to_string()
    .into_bytes();
    let outcome = member
        .call(&worker, staking.id(), "register_in_dao")
        .args(args)
        .max_gas()
        .deposit(MIN_REGISTER_DEPOSIT)
        .transact()
        .await?;
    assert!(outcome.is_success());
    outcome_pretty("register token_holder in dao", &outcome); */

    Ok(())
}
