use library::data::skyward::{AUCTION_DURATION, AUCTION_START};
use near_sdk::ONE_NEAR;
use workspaces::network::DevAccountDeployer;

use crate::contract_utils::{
    dao::{
        init::{deploy_dao, init_dao},
        proposal::{create_proposal, ps_wf_add_skyward, ts_for_skyward},
        types::{
            consts::{DAO_TPL_ID_WF_ADD, PROVIDER_TPL_ID_SKYWARD},
            proposal::{ProposalType, WorkflowAddOptions},
        },
    },
    functions::storage_deposit,
    fungible_token::init_fungible_token,
    skyward::init_skyward,
    staking::init_staking,
    wnear::init_wnear,
    workflow_provider::{init_workflow_provider, load_workflow_templates},
};

fn dao_init_args_as_bytes() -> Vec<u8> {
    vec![]
}

const DAO_FT_TOTAL_SUPPLY: u128 = 1_000_000_000;

/// Test sale create on skyward scenario as DAO with production binaries.
/// TODO: Involve factory account in the process.
/// TODO: Refactor boilerplate steps into functions.
#[tokio::test]
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
    let dao = deploy_dao(&worker).await?;
    let token = init_fungible_token(&worker, dao.id(), DAO_FT_TOTAL_SUPPLY).await?;

    // Inits dao and does essential checks.
    init_dao(
        &worker,
        &factory,
        &dao,
        token.id(),
        DAO_FT_TOTAL_SUPPLY as u32,
        24,
        staking.id(),
        wf_provider.id(),
        factory.id(),
        vec![member.id()],
    )
    .await?;

    // Storage deposit staking in fungible_token.
    let _ = storage_deposit(&worker, &factory, &token, staking.id()).await?;

    // Load workflows to provider.
    let _ = load_workflow_templates(&worker, &wf_provider, wnear.id(), skyward.id()).await?;

    // Create proposal on DAO to download Skyward workflow.
    let _ = create_proposal(
        &worker,
        &member,
        &dao,
        DAO_TPL_ID_WF_ADD,
        ps_wf_add_skyward(token.id(), 1_000, AUCTION_START, AUCTION_DURATION),
        Some(ts_for_skyward(PROVIDER_TPL_ID_SKYWARD)),
        ONE_NEAR,
    )
    .await?;

    // Vote on proposal.

    // Execute AddWorkflow by DAO member to add Skyward.

    // Propose Skyward.

    // Execute Skyward.

    // Check Skyward auction registered on DAO.
    // Check auction created on Skyward.

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
