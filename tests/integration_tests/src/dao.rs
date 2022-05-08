use near_sdk::ONE_NEAR;

use library::{
    data::workflows::{
        basic::{
            bounty::{Bounty1, Bounty1ProposeOptions},
            trade::{Trade1, Trade1ProposeOptions},
            wf_add::{WfAdd1, WfAdd1ProposeOptions},
        },
        integration::skyward::{Skyward1, Skyward1ProposeOptions, AUCTION_DURATION, AUCTION_START},
    },
    types::datatype::Value,
    workflow::instance::InstanceState,
};
use workspaces::network::DevAccountDeployer;

use crate::{
    contract_utils::{
        dao::{
            activity::{
                bounty::ActivityInputBounty1, run_activity, trade::ActivityInputTrade1,
                ActivityInputSkyward1, ActivityInputWfAdd1,
            },
            check::{check_instance, check_wf_storage_values, check_wf_templates},
            init::{deploy_dao, init_dao},
            proposal::proposal_to_finish,
            types::{
                consts::{
                    DAO_TPL_ID_OF_FIRST_ADDED, DAO_TPL_ID_WF_ADD, PROVIDER_TPL_ID_BOUNTY1,
                    PROVIDER_TPL_ID_SKYWARD1, PROVIDER_TPL_ID_TRADE1,
                },
                proposal::ProposalState,
            },
            view::{debug_log, ft_balance_of},
        },
        functions::{ft_transfer_call, serialized_dao_ft_receiver_msg, storage_deposit},
        fungible_token::init_fungible_token,
        skyward::{check_sale, init_skyward},
        staking::init_staking,
        wnear::init_wnear,
        workflow_provider::{init_workflow_provider, load_workflow_templates},
    },
    utils::Wait,
};

const DAO_FT_TOTAL_SUPPLY: u128 = 1_000_000_000;

/// Test sale create on skyward scenario as DAO with production binaries.
/// TODO: Involve factory account in the process.
#[tokio::test]
async fn workflow_skyward1_scenario() -> anyhow::Result<()> {
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
    load_workflow_templates(&worker, &wf_provider, Some(wnear.id()), Some(skyward.id())).await?;

    let proposal_id = proposal_to_finish(
        &worker,
        &member,
        &dao,
        DAO_TPL_ID_WF_ADD,
        WfAdd1::propose_settings(Some(WfAdd1ProposeOptions {
            template_id: PROVIDER_TPL_ID_SKYWARD1,
            provider_id: wf_provider.id().to_string(),
        })),
        Some(vec![Skyward1::template_settings()]),
        vec![(&member, 1)],
        100,
        WfAdd1::deposit_propose(),
        WfAdd1::deposit_vote(),
        ProposalState::Accepted,
    )
    .await?;

    // Execute AddWorkflow by DAO member to add Skyward.
    run_activity(
        &worker,
        &member,
        &dao,
        proposal_id,
        1,
        ActivityInputWfAdd1::activity_1(wf_provider.id(), PROVIDER_TPL_ID_SKYWARD1),
        true,
    )
    .await?;
    worker.wait(10).await?;
    check_wf_templates(&worker, &dao, 2).await?;
    check_instance(&worker, &dao, proposal_id, 1, InstanceState::Finished).await?;

    // Propose Skyward.
    let proposal_id = proposal_to_finish(
        &worker,
        &member,
        &dao,
        DAO_TPL_ID_OF_FIRST_ADDED,
        Skyward1::propose_settings(
            Some(Skyward1ProposeOptions {
                token_account_id: token.id().to_string(),
                token_amount: 1_000,
                auction_start: AUCTION_START,
                auction_duration: AUCTION_DURATION,
            }),
            Some("wf_skyward1"),
        ),
        None,
        vec![(&member, 1)],
        100,
        Skyward1::deposit_propose(),
        Skyward1::deposit_vote(),
        ProposalState::Accepted,
    )
    .await?;

    // Execute workflow Skyward1.
    run_activity(
        &worker,
        &member,
        &dao,
        proposal_id,
        1,
        ActivityInputSkyward1::activity_1(skyward.id()),
        true,
    )
    .await?;
    worker.wait(5).await?;

    run_activity(
        &worker,
        &member,
        &dao,
        proposal_id,
        2,
        ActivityInputSkyward1::activity_2(wnear.id(), token.id()),
        true,
    )
    .await?;
    worker.wait(5).await?;

    // Check storage
    check_wf_storage_values(
        &worker,
        &dao,
        "wf_skyward1".into(),
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
        true,
    )
    .await?;
    worker.wait(10).await?;
    check_instance(&worker, &dao, proposal_id, 3, InstanceState::Running).await?;
    ft_balance_of(&worker, &token, &skyward.id()).await?;

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
        true,
    )
    .await?;

    worker.wait(5).await?;
    debug_log(&worker, &dao).await?;

    // Check Skyward auction registered on DAO.
    check_wf_storage_values(
        &worker,
        &dao,
        "wf_skyward1".into(),
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

    /*****      Second proposal for Skyward. Skipping optional 2. activity.       *****/

    let proposal_id = proposal_to_finish(
        &worker,
        &member,
        &dao,
        DAO_TPL_ID_OF_FIRST_ADDED,
        Skyward1::propose_settings(
            Some(Skyward1ProposeOptions {
                token_account_id: token.id().to_string(),
                token_amount: 1_000,
                auction_start: AUCTION_START,
                auction_duration: AUCTION_DURATION,
            }),
            Some("wf_skyward2"),
        ),
        None,
        vec![(&member, 1)],
        100,
        Skyward1::deposit_propose(),
        Skyward1::deposit_vote(),
        ProposalState::Accepted,
    )
    .await?;

    run_activity(
        &worker,
        &member,
        &dao,
        proposal_id,
        1,
        ActivityInputSkyward1::activity_1(skyward.id()),
        true,
    )
    .await?;
    worker.wait(5).await?;

    // Check storage
    check_wf_storage_values(
        &worker,
        &dao,
        "wf_skyward2".into(),
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
        true,
    )
    .await?;
    worker.wait(10).await?;
    check_instance(&worker, &dao, proposal_id, 3, InstanceState::Running).await?;
    ft_balance_of(&worker, &token, &skyward.id()).await?;

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
        true,
    )
    .await?;

    worker.wait(5).await?;
    debug_log(&worker, &dao).await?;

    // Check Skyward auction registered on DAO.
    check_wf_storage_values(
        &worker,
        &dao,
        "wf_skyward2".into(),
        vec![
            ("pp_1_result".into(), Value::Bool(true)),
            ("pp_3_result".into(), Value::Bool(true)),
            ("skyward_auction_id".into(), Value::U64(1)),
        ],
    )
    .await?;
    check_instance(&worker, &dao, proposal_id, 4, InstanceState::Finished).await?;

    // Check auction created on Skyward.
    check_sale(
        &worker,
        &skyward,
        1,
        "NearDAO auction.".into(),
        "wwww.neardao.com".into(),
        token.id(),
        1_000,
        wnear.id(),
    )
    .await?;
    Ok(())
}

/// Sending amount of required tokens by DAO enables to send the sender NEAR tokens.
/// All values are defined in propose settings.
#[tokio::test]
async fn workflow_trade1_scenario() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let member = worker.dev_create_account().await?;
    let token_holder = worker.dev_create_account().await?;
    let registrar = worker.dev_create_account().await?;
    let factory = worker.dev_create_account().await?;

    // Contracts init.
    let staking = init_staking(&worker, registrar.id()).await?;
    let wf_provider = init_workflow_provider(&worker).await?;
    let dao = deploy_dao(&worker).await?;
    let vote_token = init_fungible_token(&worker, dao.id(), DAO_FT_TOTAL_SUPPLY).await?;
    let required_token = init_fungible_token(&worker, token_holder.id(), 1_000).await?;

    // Inits dao and does essential checks.
    init_dao(
        &worker,
        &factory,
        &dao,
        vote_token.id(),
        DAO_FT_TOTAL_SUPPLY as u32,
        24,
        staking.id(),
        wf_provider.id(),
        factory.id(),
        vec![member.id()],
    )
    .await?;

    // Storage deposit staking in fungible_token.
    storage_deposit(&worker, &factory, &vote_token, staking.id(), ONE_NEAR).await?;

    // Load workflows to provider.
    load_workflow_templates(&worker, &wf_provider, None, None).await?;

    let proposal_id = proposal_to_finish(
        &worker,
        &member,
        &dao,
        DAO_TPL_ID_WF_ADD,
        WfAdd1::propose_settings(Some(WfAdd1ProposeOptions {
            template_id: PROVIDER_TPL_ID_TRADE1,
            provider_id: wf_provider.id().to_string(),
        })),
        Some(vec![Trade1::template_settings(Some(20))]),
        vec![(&member, 1)],
        100,
        WfAdd1::deposit_propose(),
        WfAdd1::deposit_vote(),
        ProposalState::Accepted,
    )
    .await?;

    // Execute AddWorkflow by DAO member to add Trade1 template.
    run_activity(
        &worker,
        &member,
        &dao,
        proposal_id,
        1,
        ActivityInputWfAdd1::activity_1(wf_provider.id(), PROVIDER_TPL_ID_TRADE1),
        true,
    )
    .await?;
    worker.wait(10).await?;
    check_wf_templates(&worker, &dao, 2).await?;
    check_instance(&worker, &dao, proposal_id, 1, InstanceState::Finished).await?;

    // Propose Trade1.
    let proposal_id = proposal_to_finish(
        &worker,
        &member,
        &dao,
        DAO_TPL_ID_OF_FIRST_ADDED,
        Trade1::propose_settings(
            Some(Trade1ProposeOptions {
                required_token_id: required_token.id().to_string(),
                required_token_amount: 1_000,
                offered_near_amount: ONE_NEAR * 10,
            }),
            Some("wf_trade1"),
        ),
        None,
        vec![(&member, 1)],
        100,
        Trade1::deposit_propose(),
        Trade1::deposit_vote(),
        ProposalState::Accepted,
    )
    .await?;

    // Storage deposit dao in required token contract.
    storage_deposit(&worker, &token_holder, &required_token, dao.id(), ONE_NEAR).await?;

    // Transfer tokens to make trade.
    ft_transfer_call(
        &worker,
        &token_holder,
        &required_token,
        &dao.id(),
        1_000,
        None,
        serialized_dao_ft_receiver_msg(2),
    )
    .await?;
    debug_log(&worker, &dao).await?;
    check_wf_storage_values(&worker, &dao, "wf_trade1".into(), vec![]).await?;
    let dao_account_balance_before = dao.view_account(&worker).await?.balance / 10u128.pow(24);

    // Execute workflow Trade1.
    run_activity(
        &worker,
        &member,
        &dao,
        proposal_id,
        1,
        ActivityInputTrade1::activity_1(),
        true,
    )
    .await?;
    worker.wait(5).await?;
    assert_eq!(
        ft_balance_of(&worker, &required_token, &token_holder.id()).await?,
        0.into()
    );
    assert_eq!(
        ft_balance_of(&worker, &required_token, &dao.id()).await?,
        1_000.into()
    );
    let dao_account_balance_after = dao.view_account(&worker).await?.balance / 10u128.pow(24);
    println!(
        "dao balance before/after: {},{}",
        dao_account_balance_before, dao_account_balance_after
    );
    assert_eq!(dao_account_balance_before - 10, dao_account_balance_after);
    debug_log(&worker, &dao).await?;
    check_instance(&worker, &dao, proposal_id, 1, InstanceState::Finished).await?;
    Ok(())
}

/// Activity is not executed because invalid token was send.
#[tokio::test]
async fn workflow_trade1_invalid_token() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let member = worker.dev_create_account().await?;
    let token_holder = worker.dev_create_account().await?;
    let registrar = worker.dev_create_account().await?;
    let factory = worker.dev_create_account().await?;

    // Contracts init.
    let staking = init_staking(&worker, registrar.id()).await?;
    let wf_provider = init_workflow_provider(&worker).await?;
    let dao = deploy_dao(&worker).await?;
    let vote_token = init_fungible_token(&worker, dao.id(), DAO_FT_TOTAL_SUPPLY).await?;
    let required_token = init_fungible_token(&worker, token_holder.id(), 1_000).await?;
    let other_token = init_fungible_token(&worker, token_holder.id(), 1_000).await?;

    // Inits dao and does essential checks.
    init_dao(
        &worker,
        &factory,
        &dao,
        vote_token.id(),
        DAO_FT_TOTAL_SUPPLY as u32,
        24,
        staking.id(),
        wf_provider.id(),
        factory.id(),
        vec![member.id()],
    )
    .await?;

    // Storage deposit staking in fungible_token.
    storage_deposit(&worker, &factory, &vote_token, staking.id(), ONE_NEAR).await?;

    // Load workflows to provider.
    load_workflow_templates(&worker, &wf_provider, None, None).await?;

    let proposal_id = proposal_to_finish(
        &worker,
        &member,
        &dao,
        DAO_TPL_ID_WF_ADD,
        WfAdd1::propose_settings(Some(WfAdd1ProposeOptions {
            template_id: PROVIDER_TPL_ID_TRADE1,
            provider_id: wf_provider.id().to_string(),
        })),
        Some(vec![Trade1::template_settings(Some(20))]),
        vec![(&member, 1)],
        100,
        WfAdd1::deposit_propose(),
        WfAdd1::deposit_vote(),
        ProposalState::Accepted,
    )
    .await?;

    // Execute AddWorkflow by DAO member to add Trade1 template.
    run_activity(
        &worker,
        &member,
        &dao,
        proposal_id,
        1,
        ActivityInputWfAdd1::activity_1(wf_provider.id(), PROVIDER_TPL_ID_TRADE1),
        true,
    )
    .await?;
    worker.wait(10).await?;
    check_wf_templates(&worker, &dao, 2).await?;
    check_instance(&worker, &dao, proposal_id, 1, InstanceState::Finished).await?;

    // Propose Trade1.
    let proposal_id = proposal_to_finish(
        &worker,
        &member,
        &dao,
        DAO_TPL_ID_OF_FIRST_ADDED,
        Trade1::propose_settings(
            Some(Trade1ProposeOptions {
                required_token_id: required_token.id().to_string(),
                required_token_amount: 1_000,
                offered_near_amount: ONE_NEAR * 10,
            }),
            Some("wf_trade1"),
        ),
        None,
        vec![(&member, 1)],
        100,
        Trade1::deposit_propose(),
        Trade1::deposit_vote(),
        ProposalState::Accepted,
    )
    .await?;

    // Storage deposit dao in required token contract.
    storage_deposit(&worker, &token_holder, &other_token, dao.id(), ONE_NEAR).await?;

    // Transfer tokens to make trade.
    ft_transfer_call(
        &worker,
        &token_holder,
        &other_token,
        &dao.id(),
        1_000,
        None,
        serialized_dao_ft_receiver_msg(proposal_id),
    )
    .await?;
    debug_log(&worker, &dao).await?;
    check_wf_storage_values(&worker, &dao, "wf_trade1".into(), vec![]).await?;
    let dao_account_balance_before = dao.view_account(&worker).await?.balance / 10u128.pow(24);

    // Execute workflow Trade1.
    run_activity(
        &worker,
        &member,
        &dao,
        proposal_id,
        1,
        ActivityInputTrade1::activity_1(),
        false,
    )
    .await?;
    worker.wait(5).await?;
    assert_eq!(
        ft_balance_of(&worker, &required_token, &token_holder.id()).await?,
        1_000.into()
    );
    assert_eq!(
        ft_balance_of(&worker, &required_token, &dao.id()).await?,
        0.into()
    );
    assert_eq!(
        ft_balance_of(&worker, &other_token, &token_holder.id()).await?,
        0.into()
    );
    assert_eq!(
        ft_balance_of(&worker, &other_token, &dao.id()).await?,
        1_000.into()
    );
    let dao_account_balance_after = dao.view_account(&worker).await?.balance / 10u128.pow(24);
    println!(
        "dao balance before/after: {},{}",
        dao_account_balance_before, dao_account_balance_after
    );
    assert_eq!(dao_account_balance_before, dao_account_balance_after);
    debug_log(&worker, &dao).await?;
    check_instance(&worker, &dao, proposal_id, 0, InstanceState::Running).await?;
    Ok(())
}

/// Make bounty and confirm the task was done and send them NEAR as a reward.
/// All values are defined in propose settings.
#[tokio::test]
async fn workflow_bounty1_scenario() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let member = worker.dev_create_account().await?;
    let bounty_hunter = worker.dev_create_account().await?;
    let registrar = worker.dev_create_account().await?;
    let factory = worker.dev_create_account().await?;

    // Contracts init.
    let staking = init_staking(&worker, registrar.id()).await?;
    let wf_provider = init_workflow_provider(&worker).await?;
    let dao = deploy_dao(&worker).await?;
    let vote_token = init_fungible_token(&worker, dao.id(), DAO_FT_TOTAL_SUPPLY).await?;

    // Inits dao and does essential checks.
    init_dao(
        &worker,
        &factory,
        &dao,
        vote_token.id(),
        DAO_FT_TOTAL_SUPPLY as u32,
        24,
        staking.id(),
        wf_provider.id(),
        factory.id(),
        vec![member.id()],
    )
    .await?;

    // Storage deposit staking in fungible_token.
    storage_deposit(&worker, &factory, &vote_token, staking.id(), ONE_NEAR).await?;

    // Load workflows to provider.
    load_workflow_templates(&worker, &wf_provider, None, None).await?;

    let proposal_id = proposal_to_finish(
        &worker,
        &member,
        &dao,
        DAO_TPL_ID_WF_ADD,
        WfAdd1::propose_settings(Some(WfAdd1ProposeOptions {
            template_id: PROVIDER_TPL_ID_BOUNTY1,
            provider_id: wf_provider.id().to_string(),
        })),
        Some(vec![Bounty1::template_settings(Some(20))]),
        vec![(&member, 1)],
        100,
        WfAdd1::deposit_propose(),
        WfAdd1::deposit_vote(),
        ProposalState::Accepted,
    )
    .await?;

    // Execute AddWorkflow by DAO member to add Bounty1 template.
    run_activity(
        &worker,
        &member,
        &dao,
        proposal_id,
        1,
        ActivityInputWfAdd1::activity_1(wf_provider.id(), PROVIDER_TPL_ID_BOUNTY1),
        true,
    )
    .await?;
    worker.wait(10).await?;
    check_wf_templates(&worker, &dao, 2).await?;
    check_instance(&worker, &dao, proposal_id, 1, InstanceState::Finished).await?;

    // Propose Bounty1.
    let proposal_id = proposal_to_finish(
        &worker,
        &member,
        &dao,
        DAO_TPL_ID_OF_FIRST_ADDED,
        Bounty1::propose_settings(
            Some(Bounty1ProposeOptions {
                max_offered_near_amount: ONE_NEAR * 10,
            }),
            Some("wf_bounty1"),
        ),
        None,
        vec![(&member, 1)],
        100,
        Bounty1::deposit_propose(),
        Bounty1::deposit_vote(),
        ProposalState::Accepted,
    )
    .await?;

    // Execute workflow Bounty1.
    run_activity(
        &worker,
        &bounty_hunter,
        &dao,
        proposal_id,
        1,
        ActivityInputBounty1::activity_1(),
        true,
    )
    .await?;
    worker.wait(5).await?;
    check_wf_storage_values(&worker, &dao, "wf_bounty1".into(), vec![]).await?;
    run_activity(
        &worker,
        &member,
        &dao,
        proposal_id,
        3,
        ActivityInputBounty1::activity_3(true),
        true,
    )
    .await?;
    worker.wait(5).await?;
    check_wf_storage_values(&worker, &dao, "wf_bounty1".into(), vec![]).await?;
    debug_log(&worker, &dao).await?;
    run_activity(
        &worker,
        &bounty_hunter,
        &dao,
        proposal_id,
        4,
        ActivityInputBounty1::activity_4("here link blabla..".into()),
        true,
    )
    .await?;
    worker.wait(5).await?;
    check_wf_storage_values(&worker, &dao, "wf_bounty1".into(), vec![]).await?;
    run_activity(
        &worker,
        &member,
        &dao,
        proposal_id,
        5,
        ActivityInputBounty1::activity_5("perfect - 10/10".into()),
        true,
    )
    .await?;
    worker.wait(5).await?;
    let dao_account_balance_before = dao.view_account(&worker).await?.balance / 10u128.pow(24);
    let bounty_hunter_balance_before =
        bounty_hunter.view_account(&worker).await?.balance / 10u128.pow(24);
    check_wf_storage_values(&worker, &dao, "wf_bounty1".into(), vec![]).await?;
    debug_log(&worker, &dao).await?;
    run_activity(
        &worker,
        &member,
        &dao,
        proposal_id,
        6,
        ActivityInputBounty1::activity_6(&bounty_hunter.id(), ONE_NEAR * 10),
        true,
    )
    .await?;
    worker.wait(5).await?;

    let dao_account_balance_after = dao.view_account(&worker).await?.balance / 10u128.pow(24);
    let bounty_hunter_balance_after =
        bounty_hunter.view_account(&worker).await?.balance / 10u128.pow(24);
    assert_eq!(dao_account_balance_before - 10, dao_account_balance_after);
    assert_eq!(
        bounty_hunter_balance_before + 10,
        bounty_hunter_balance_after
    );
    debug_log(&worker, &dao).await?;
    check_instance(&worker, &dao, proposal_id, 6, InstanceState::Finished).await?;
    Ok(())
}
