use library::{
    data::skyward::{AUCTION_DURATION, AUCTION_START},
    workflow::instance::InstanceState,
};
use workspaces::network::DevAccountDeployer;

use crate::contract_utils::{
    dao::{
        activity::{run_activity, ActivityInputWorkflowAdd},
        init::{deploy_dao, init_dao},
        proposal::{
            create_proposal, finish_proposal, ps_skyward, ps_wf_add, ts_for_skyward, vote_proposal,
        },
        types::{
            consts::{
                DAO_TPL_ID_WF_ADD, DEPOSIT_PROPOSE_WF_ADD, DEPOSIT_VOTE_WF_ADD,
                PROVIDER_TPL_ID_SKYWARD,
            },
            proposal::ProposalState,
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
    storage_deposit(&worker, &factory, &token, staking.id()).await?;

    // Load workflows to provider.
    load_workflow_templates(&worker, &wf_provider, wnear.id(), skyward.id()).await?;

    // Create proposal on DAO to download Skyward workflow.
    let proposal_id = create_proposal(
        &worker,
        &member,
        &dao,
        DAO_TPL_ID_WF_ADD,
        ps_wf_add(PROVIDER_TPL_ID_SKYWARD, wf_provider.id()),
        Some(ts_for_skyward()),
        DEPOSIT_PROPOSE_WF_ADD,
    )
    .await?;

    // Vote on proposal.
    vote_proposal(
        &worker,
        vec![(&member, 1)],
        &dao,
        proposal_id,
        DEPOSIT_VOTE_WF_ADD,
    )
    .await?;

    // Finish proposal.
    finish_proposal(
        &worker,
        &member,
        &dao,
        proposal_id,
        ProposalState::InProgress,
    )
    .await?;

    // Fast forward and finish.
    worker.fast_forward(100).await?;
    finish_proposal(&worker, &member, &dao, proposal_id, ProposalState::Accepted).await?;

    // Execute AddWorkflow by DAO member to add Skyward.
    run_activity(
        &worker,
        &member,
        &dao,
        proposal_id,
        1,
        ActivityInputWorkflowAdd::activity_1(wf_provider.id(), PROVIDER_TPL_ID_SKYWARD),
        InstanceState::Running,
    )
    .await?;

    // Propose Skyward.
    /*     let proposal_id = create_proposal(
        &worker,
        &member,
        &dao,
        DAO_TPL_ID_WF_ADD,
        ps_skyward(token.id(), 1_000, AUCTION_START, AUCTION_DURATION),
        Some(ts_for_skyward()),
        DEPOSIT_PROPOSE_WF_ADD,
    )
    .await?; */

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
