use near_sdk::ONE_NEAR;
#[allow(unused)]
use std::time::Duration;
#[allow(unused)]
use tokio::time::sleep;

use library::{
    data::workflows::{
        basic::wf_add::{WfAdd1, WfAdd1ProposeOptions},
        integration::skyward::{
            Skyward1, Skyward1ProposeOptions, AUCTION_DURATION, AUCTION_START, SKYWARD1_STORAGE_KEY,
        },
    },
    types::datatype::Value,
    workflow::instance::InstanceState,
};
use workspaces::network::{DevAccountDeployer, Sandbox};

use crate::contract_utils::{
    dao::{
        activity::{run_activity, ActivityInputSkyward1, ActivityInputWfAdd1},
        check::{check_instance, check_wf_storage_values, check_wf_templates},
        init::{deploy_dao, init_dao},
        proposal::{create_proposal, finish_proposal, vote_proposal},
        types::{
            consts::{DAO_TPL_ID_SKYWARD, DAO_TPL_ID_WF_ADD, PROVIDER_TPL_ID_SKYWARD},
            proposal::ProposalState,
        },
        view::{debug_log, ft_balance_of},
    },
    functions::storage_deposit,
    fungible_token::init_fungible_token,
    skyward::{check_sale, init_skyward},
    staking::init_staking,
    wnear::init_wnear,
    workflow_provider::{init_workflow_provider, load_workflow_templates},
};

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
    storage_deposit(&worker, &factory, &token, staking.id(), ONE_NEAR).await?;

    // Load workflows to provider.
    load_workflow_templates(&worker, &wf_provider, wnear.id(), skyward.id()).await?;

    // Create proposal on DAO to download Skyward workflow.
    let proposal_id = create_proposal(
        &worker,
        &member,
        &dao,
        DAO_TPL_ID_WF_ADD,
        WfAdd1::propose_settings(Some(WfAdd1ProposeOptions {
            template_id: PROVIDER_TPL_ID_SKYWARD,
            provider_id: wf_provider.id().to_string(),
        })),
        Some(Skyward1::template_settings()),
        WfAdd1::deposit_propose(),
    )
    .await?;

    // Vote on proposal.
    vote_proposal(
        &worker,
        vec![(&member, 1)],
        &dao,
        proposal_id,
        WfAdd1::deposit_vote(),
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
    //sleep(Duration::from_secs(120)).await;
    finish_proposal(&worker, &member, &dao, proposal_id, ProposalState::Accepted).await?;

    // Execute AddWorkflow by DAO member to add Skyward.
    run_activity(
        &worker,
        &member,
        &dao,
        proposal_id,
        1,
        ActivityInputWfAdd1::activity_1(wf_provider.id(), PROVIDER_TPL_ID_SKYWARD),
    )
    .await?;
    worker.fast_forward(10).await?;
    //sleep(Duration::from_secs(10)).await;
    check_wf_templates(&worker, &dao, 2).await?;
    check_instance(&worker, &dao, proposal_id, 1, InstanceState::Finished).await?;

    // Propose Skyward.
    let proposal_id = create_proposal(
        &worker,
        &member,
        &dao,
        DAO_TPL_ID_SKYWARD,
        Skyward1::propose_settings(
            Some(Skyward1ProposeOptions {
                token_account_id: token.id().to_string(),
                token_amount: 1_000,
                auction_start: AUCTION_START,
                auction_duration: AUCTION_DURATION,
            }),
            None,
        ),
        None,
        Skyward1::deposit_propose(),
    )
    .await?;

    // Vote on proposed Skyward.
    vote_proposal(
        &worker,
        vec![(&member, 1)],
        &dao,
        proposal_id,
        Skyward1::deposit_vote(),
    )
    .await?;

    // Finish last proposal.
    worker.fast_forward(100).await?;
    //sleep(Duration::from_secs(120)).await;
    finish_proposal(&worker, &member, &dao, proposal_id, ProposalState::Accepted).await?;

    // Execute workflow Skyward1.
    run_activity(
        &worker,
        &member,
        &dao,
        proposal_id,
        1,
        ActivityInputSkyward1::activity_1(skyward.id()),
    )
    .await?;
    //sleep(Duration::from_secs(10)).await;
    worker.fast_forward(5).await?;

    run_activity(
        &worker,
        &member,
        &dao,
        proposal_id,
        2,
        ActivityInputSkyward1::activity_2(wnear.id(), token.id()),
    )
    .await?;
    //sleep(Duration::from_secs(10)).await;
    worker.fast_forward(5).await?;

    // Check storage
    check_wf_storage_values(
        &worker,
        &dao,
        SKYWARD1_STORAGE_KEY.into(),
        vec![("pp_1_result".into(), Value::Bool(true))],
    )
    .await?;
    debug_log(&worker, &dao).await?;
    run_activity(
        &worker,
        &member,
        &dao,
        proposal_id,
        3,
        ActivityInputSkyward1::activity_3(token.id()),
    )
    .await?;
    //sleep(Duration::from_secs(10)).await;
    worker.fast_forward(10).await?;
    //check_instance(&worker, &dao, proposal_id, 3, InstanceState::Running).await?;
    ft_balance_of(&worker, &token, &skyward.id()).await?;

    /*     // Check storage
    check_wf_storage_values(
        &worker,
        &dao,
        SKYWARD1_STORAGE_KEY.into(),
        vec![
            ("pp_1_result".into(), Value::Bool(true)),
            ("pp_3_result".into(), Value::Bool(true)),
        ],
    )
    .await?; */

    debug_log(&worker, &dao).await?;

    run_activity(
        &worker,
        &member,
        &dao,
        proposal_id,
        4,
        ActivityInputSkyward1::activity_4(
            skyward.id(),
            "NearDAO auction.".into(),
            "wwww.neardao.com".into(),
        ),
    )
    .await?;
    //sleep(Duration::from_secs(10)).await;
    worker.fast_forward(5).await?;
    debug_log(&worker, &dao).await?;

    // Check Skyward auction registered on DAO.
    check_wf_storage_values(
        &worker,
        &dao,
        SKYWARD1_STORAGE_KEY.into(),
        vec![
            ("pp_1_result".into(), Value::Bool(true)),
            ("pp_3_result".into(), Value::Bool(true)),
            ("skyward_auction_id".into(), Value::U64(0)),
        ],
    )
    .await?;
    check_instance(&worker, &dao, proposal_id, 4, InstanceState::Finished).await?;

    // Check auction created on Skyward.
    check_sale(
        &worker,
        &skyward,
        0,
        "NearDAO auction.".into(),
        "wwww.neardao.com".into(),
        token.id(),
        1_000,
        wnear.id(),
    )
    .await?;

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
