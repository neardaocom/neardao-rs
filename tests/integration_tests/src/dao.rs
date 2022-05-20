use near_sdk::ONE_NEAR;

use crate::contract_utils::dao::{
    activity_input::{
        ActivityInputAdminPkg1, ADMINPACKAGE1_ADD_GROUP, ADMINPACKAGE1_ADD_GROUP_MEMBERS,
        ADMINPACKAGE1_REMOVE_GROUP, ADMINPACKAGE1_REMOVE_GROUP_MEMBERS,
        ADMINPACKAGE1_REMOVE_GROUP_MEMBER_ROLES, ADMINPACKAGE1_REMOVE_GROUP_ROLES,
    },
    check::{check_group, check_group_exists, check_group_roles, check_user_roles},
    types::{
        consts::PROVIDER_TPL_ID_ADMIN_PACKAGE1,
        group::{Roles, UserRoles},
    },
    view::{statistics, view_groups, view_partitions},
};
#[allow(unused)]
use crate::{
    contract_utils::{
        dao::{
            activity_input::{
                run_activity, ActivityInputBounty1, ActivityInputReward1, ActivityInputSkyward1,
                ActivityInputTrade1, ActivityInputWfAdd1,
            },
            check::{check_instance, check_wf_storage_values, check_wf_templates},
            proposal::{proposal_to_finish, proposal_to_finish_testnet},
            reward::withdraw_rewards,
            types::{
                consts::{
                    DAO_TPL_ID_OF_FIRST_ADDED, DAO_TPL_ID_WF_ADD, PROVIDER_TPL_ID_BOUNTY1,
                    PROVIDER_TPL_ID_REWARD1, PROVIDER_TPL_ID_SKYWARD1, PROVIDER_TPL_ID_TRADE1,
                },
                proposal::ProposalState,
                reward::{Asset, RewardActivity},
            },
            view::{
                debug_log, ft_balance_of, get_timestamp, view_reward, view_user_roles,
                view_user_wallet,
            },
        },
        dao_factory::{create_dao_via_factory, init_dao_factory},
        ft_factory::{create_ft_via_factory, init_ft_factory},
        functions::{ft_transfer_call, serialized_dao_ft_receiver_msg, storage_deposit},
        skyward::{check_sale, init_skyward},
        staking::init_staking,
        wnear::init_wnear,
        workflow_provider::{init_workflow_provider, load_workflow_templates},
    },
    utils::Wait,
};
use data::workflow::{
    basic::{
        admin_package::AdminPackage1,
        bounty::{Bounty1, Bounty1ProposeOptions},
        reward::Reward1,
        trade::{Trade1, Trade1ProposeOptions},
        wf_add::{WfAdd1, WfAdd1ProposeOptions},
    },
    integration::skyward::{Skyward1, Skyward1ProposeOptions, AUCTION_DURATION, AUCTION_START},
};
use library::{types::datatype::Value, workflow::instance::InstanceState};
use workspaces::{network::DevAccountDeployer, AccountId};

const DAO_FT_TOTAL_SUPPLY: u128 = 1_000_000_000;
const DEFAULT_DECIMALS: u128 = 10u128.pow(24);

/// Test sale create on skyward scenario as DAO with production binaries.
/// TODO: Involve factory account in the process.
#[tokio::test]
async fn workflow_skyward1_scenario() -> anyhow::Result<()> {
    let dao_name = "test_dao";
    let ft_name = "dao_token";
    let worker = workspaces::sandbox().await?;
    let member = worker.dev_create_account().await?;

    // Contracts init.
    let ft_factory = init_ft_factory(&worker).await?;
    let factory = init_dao_factory(&worker).await?;
    let dao_account_id = AccountId::try_from(format!("{}.{}", dao_name, factory.id()))
        .expect("invalid dao account id");
    let token_account_id = AccountId::try_from(format!("{}.{}", ft_name, ft_factory.id()))
        .expect("invalid ft account id");
    create_ft_via_factory(
        &worker,
        &ft_factory,
        ft_name,
        dao_account_id.as_str(),
        DAO_FT_TOTAL_SUPPLY,
        24,
        None,
        None,
        vec![],
    )
    .await?;

    let staking = init_staking(&worker).await?;
    let wf_provider = init_workflow_provider(&worker).await?;
    create_dao_via_factory(
        &worker,
        &factory,
        &dao_name,
        &token_account_id,
        DAO_FT_TOTAL_SUPPLY as u32,
        24,
        staking.id(),
        wf_provider.id(),
        factory.id(),
        vec![member.id()],
        0,
    )
    .await?;
    let wnear = init_wnear(&worker).await?;
    let skyward = init_skyward(&worker, &wnear, None).await?;
    let user_roles = UserRoles::new().add_group_roles(1, vec![0, 1]);
    let group_roles = Roles::new().add_role("council");
    check_user_roles(&worker, &dao_account_id, member.id(), Some(&user_roles)).await?;
    check_group_roles(&worker, &dao_account_id, 1, &group_roles).await?;

    statistics(&worker, &dao_account_id).await?;

    // Storage deposit staking in fungible_token.
    storage_deposit(
        &worker,
        factory.as_account(),
        &token_account_id,
        staking.id(),
        ONE_NEAR,
    )
    .await?;
    // Load workflows to provider.
    load_workflow_templates(&worker, &wf_provider, Some(wnear.id()), Some(skyward.id())).await?;

    let proposal_id = proposal_to_finish(
        &worker,
        &member,
        &dao_account_id,
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
        &dao_account_id,
        proposal_id,
        1,
        ActivityInputWfAdd1::activity_1(wf_provider.id(), PROVIDER_TPL_ID_SKYWARD1),
        true,
    )
    .await?;
    worker.wait(10).await?;
    check_wf_templates(&worker, &dao_account_id, 2).await?;
    check_instance(
        &worker,
        &dao_account_id,
        proposal_id,
        1,
        InstanceState::Finished,
    )
    .await?;

    // Propose Skyward.
    let proposal_id = proposal_to_finish(
        &worker,
        &member,
        &dao_account_id,
        DAO_TPL_ID_OF_FIRST_ADDED,
        Skyward1::propose_settings(
            Some(Skyward1ProposeOptions {
                token_account_id: token_account_id.to_string(),
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
        &dao_account_id,
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
        &dao_account_id,
        proposal_id,
        2,
        ActivityInputSkyward1::activity_2(wnear.id(), &token_account_id),
        true,
    )
    .await?;
    worker.wait(5).await?;

    // Check storage
    check_wf_storage_values(
        &worker,
        &dao_account_id,
        "wf_skyward1".into(),
        vec![("pp_1_result".into(), Value::Bool(true))],
    )
    .await?;
    debug_log(&worker, &dao_account_id).await?;
    run_activity(
        &worker,
        &member,
        &dao_account_id,
        proposal_id,
        3,
        ActivityInputSkyward1::activity_3(&token_account_id),
        true,
    )
    .await?;
    worker.wait(10).await?;
    check_instance(
        &worker,
        &dao_account_id,
        proposal_id,
        3,
        InstanceState::Running,
    )
    .await?;
    ft_balance_of(&worker, &token_account_id, &skyward.id()).await?;

    debug_log(&worker, &dao_account_id).await?;

    run_activity(
        &worker,
        &member,
        &dao_account_id,
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
    debug_log(&worker, &dao_account_id).await?;

    // Check Skyward auction registered on DAO.
    check_wf_storage_values(
        &worker,
        &dao_account_id,
        "wf_skyward1".into(),
        vec![
            ("pp_1_result".into(), Value::Bool(true)),
            ("pp_3_result".into(), Value::Bool(true)),
            ("skyward_auction_id".into(), Value::U64(0)),
        ],
    )
    .await?;
    check_instance(
        &worker,
        &dao_account_id,
        proposal_id,
        4,
        InstanceState::Finished,
    )
    .await?;

    // Check auction created on Skyward.
    check_sale(
        &worker,
        &skyward,
        0,
        "NearDAO auction.".into(),
        "wwww.neardao.com".into(),
        &token_account_id,
        1_000,
        wnear.id(),
    )
    .await?;

    /*****      Second proposal for Skyward. Skipping optional 2. activity.       *****/

    let proposal_id = proposal_to_finish(
        &worker,
        &member,
        &dao_account_id,
        DAO_TPL_ID_OF_FIRST_ADDED,
        Skyward1::propose_settings(
            Some(Skyward1ProposeOptions {
                token_account_id: token_account_id.to_string(),
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
        &dao_account_id,
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
        &dao_account_id,
        "wf_skyward2".into(),
        vec![("pp_1_result".into(), Value::Bool(true))],
    )
    .await?;
    debug_log(&worker, &dao_account_id).await?;
    run_activity(
        &worker,
        &member,
        &dao_account_id,
        proposal_id,
        3,
        ActivityInputSkyward1::activity_3(&token_account_id),
        true,
    )
    .await?;
    worker.wait(10).await?;
    check_instance(
        &worker,
        &dao_account_id,
        proposal_id,
        3,
        InstanceState::Running,
    )
    .await?;
    ft_balance_of(&worker, &token_account_id, &skyward.id()).await?;

    debug_log(&worker, &dao_account_id).await?;

    run_activity(
        &worker,
        &member,
        &dao_account_id,
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
    debug_log(&worker, &dao_account_id).await?;

    // Check Skyward auction registered on DAO.
    check_wf_storage_values(
        &worker,
        &dao_account_id,
        "wf_skyward2".into(),
        vec![
            ("pp_1_result".into(), Value::Bool(true)),
            ("pp_3_result".into(), Value::Bool(true)),
            ("skyward_auction_id".into(), Value::U64(1)),
        ],
    )
    .await?;
    check_instance(
        &worker,
        &dao_account_id,
        proposal_id,
        4,
        InstanceState::Finished,
    )
    .await?;

    // Check auction created on Skyward.
    check_sale(
        &worker,
        &skyward,
        1,
        "NearDAO auction.".into(),
        "wwww.neardao.com".into(),
        &token_account_id,
        1_000,
        wnear.id(),
    )
    .await?;
    statistics(&worker, &dao_account_id).await?;
    Ok(())
}

/// Sending amount of required tokens by DAO enables to send the sender NEAR tokens.
/// All values are defined in propose settings.
#[tokio::test]
async fn workflow_trade1_scenario() -> anyhow::Result<()> {
    let ft_name = "dao_token";
    let required_token_name = "required_token";
    let dao_name = "test_dao";
    let worker = workspaces::sandbox().await?;
    let member = worker.dev_create_account().await?;
    let token_holder = worker.dev_create_account().await?;

    // Contracts init.
    let ft_factory = init_ft_factory(&worker).await?;
    let factory = init_dao_factory(&worker).await?;
    let dao_account_id = AccountId::try_from(format!("{}.{}", dao_name, factory.id()))
        .expect("invalid dao account id");
    let token_account_id = AccountId::try_from(format!("{}.{}", ft_name, ft_factory.id()))
        .expect("invalid ft account id");
    let required_token_account_id =
        AccountId::try_from(format!("{}.{}", required_token_name, ft_factory.id()))
            .expect("invalid ft account id");
    create_ft_via_factory(
        &worker,
        &ft_factory,
        ft_name,
        dao_account_id.as_str(),
        DAO_FT_TOTAL_SUPPLY,
        24,
        None,
        None,
        vec![],
    )
    .await?;
    create_ft_via_factory(
        &worker,
        &ft_factory,
        required_token_name,
        token_holder.id().as_str(),
        1_000,
        24,
        None,
        None,
        vec![],
    )
    .await?;
    let staking = init_staking(&worker).await?;
    let wf_provider = init_workflow_provider(&worker).await?;
    create_dao_via_factory(
        &worker,
        &factory,
        &dao_name,
        &token_account_id,
        DAO_FT_TOTAL_SUPPLY as u32,
        24,
        staking.id(),
        wf_provider.id(),
        factory.id(),
        vec![member.id()],
        0,
    )
    .await?;
    //let vote_token = init_fungible_token(&worker, dao_account_id.as_str(), 1_000_000_000).await?;
    //let required_token = init_fungible_token(&worker, token_holder.id(), 1_000).await?;

    // Storage deposit staking in fungible_token.
    storage_deposit(
        &worker,
        &factory.as_account(),
        &token_account_id,
        staking.id(),
        ONE_NEAR,
    )
    .await?;

    // Load workflows to provider.
    load_workflow_templates(&worker, &wf_provider, None, None).await?;

    let proposal_id = proposal_to_finish(
        &worker,
        &member,
        &dao_account_id,
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
        &dao_account_id,
        proposal_id,
        1,
        ActivityInputWfAdd1::activity_1(wf_provider.id(), PROVIDER_TPL_ID_TRADE1),
        true,
    )
    .await?;
    worker.wait(10).await?;
    check_wf_templates(&worker, &dao_account_id, 2).await?;
    check_instance(
        &worker,
        &dao_account_id,
        proposal_id,
        1,
        InstanceState::Finished,
    )
    .await?;

    // Propose Trade1.
    let proposal_id = proposal_to_finish(
        &worker,
        &member,
        &dao_account_id,
        DAO_TPL_ID_OF_FIRST_ADDED,
        Trade1::propose_settings(
            Some(Trade1ProposeOptions {
                required_token_id: required_token_account_id.to_string(),
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
    storage_deposit(
        &worker,
        &token_holder,
        &required_token_account_id,
        &dao_account_id,
        ONE_NEAR,
    )
    .await?;

    // Transfer tokens to make trade.
    ft_transfer_call(
        &worker,
        &token_holder,
        &required_token_account_id,
        &dao_account_id,
        1_000,
        None,
        serialized_dao_ft_receiver_msg(2),
    )
    .await?;
    debug_log(&worker, &dao_account_id).await?;
    check_wf_storage_values(&worker, &dao_account_id, "wf_trade1".into(), vec![]).await?;
    let dao_account_balance_before =
        worker.view_account(&dao_account_id).await?.balance / 10u128.pow(24);

    // Execute workflow Trade1.
    run_activity(
        &worker,
        &member,
        &dao_account_id,
        proposal_id,
        1,
        ActivityInputTrade1::activity_1(),
        true,
    )
    .await?;
    worker.wait(5).await?;
    assert_eq!(
        ft_balance_of(&worker, &required_token_account_id, &token_holder.id()).await?,
        (1_000 * DEFAULT_DECIMALS - 1_000).into()
    );
    assert_eq!(
        ft_balance_of(&worker, &required_token_account_id, &dao_account_id).await?,
        (1_000).into()
    );
    let dao_account_balance_after =
        worker.view_account(&dao_account_id).await?.balance / 10u128.pow(24);
    println!(
        "dao balance before/after: {},{}",
        dao_account_balance_before, dao_account_balance_after
    );
    assert_eq!(dao_account_balance_before - 10, dao_account_balance_after);
    debug_log(&worker, &dao_account_id).await?;
    check_instance(
        &worker,
        &dao_account_id,
        proposal_id,
        1,
        InstanceState::Finished,
    )
    .await?;
    Ok(())
}

/// Activity is not executed because invalid token was send.
#[tokio::test]
async fn workflow_trade1_invalid_token() -> anyhow::Result<()> {
    let ft_name = "dao_token";
    let dao_name = "test_dao";
    let required_token_name = "required_token";
    let other_token_name = "other_token";
    let worker = workspaces::sandbox().await?;
    let member = worker.dev_create_account().await?;
    let token_holder = worker.dev_create_account().await?;

    // Contracts init.
    let ft_factory = init_ft_factory(&worker).await?;
    let factory = init_dao_factory(&worker).await?;
    let dao_account_id = AccountId::try_from(format!("{}.{}", dao_name, factory.id()))
        .expect("invalid dao account id");
    let token_account_id = AccountId::try_from(format!("{}.{}", ft_name, ft_factory.id()))
        .expect("invalid ft account id");
    let required_token_account_id =
        AccountId::try_from(format!("{}.{}", required_token_name, ft_factory.id()))
            .expect("invalid ft account id");
    let other_token_account_id =
        AccountId::try_from(format!("{}.{}", other_token_name, ft_factory.id()))
            .expect("invalid ft account id");
    create_ft_via_factory(
        &worker,
        &ft_factory,
        ft_name,
        dao_account_id.as_str(),
        DAO_FT_TOTAL_SUPPLY,
        24,
        None,
        None,
        vec![],
    )
    .await?;
    create_ft_via_factory(
        &worker,
        &ft_factory,
        required_token_name,
        token_holder.id().as_str(),
        1_000,
        24,
        None,
        None,
        vec![],
    )
    .await?;
    create_ft_via_factory(
        &worker,
        &ft_factory,
        other_token_name,
        token_holder.id().as_str(),
        1_000,
        24,
        None,
        None,
        vec![],
    )
    .await?;
    let staking = init_staking(&worker).await?;
    let wf_provider = init_workflow_provider(&worker).await?;
    create_dao_via_factory(
        &worker,
        &factory,
        &dao_name,
        &token_account_id,
        DAO_FT_TOTAL_SUPPLY as u32,
        24,
        staking.id(),
        wf_provider.id(),
        factory.id(),
        vec![member.id()],
        0,
    )
    .await?;

    // Storage deposit staking in fungible_token.
    storage_deposit(
        &worker,
        &factory.as_account(),
        &token_account_id,
        staking.id(),
        ONE_NEAR,
    )
    .await?;

    // Load workflows to provider.
    load_workflow_templates(&worker, &wf_provider, None, None).await?;

    let proposal_id = proposal_to_finish(
        &worker,
        &member,
        &dao_account_id,
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
        &dao_account_id,
        proposal_id,
        1,
        ActivityInputWfAdd1::activity_1(wf_provider.id(), PROVIDER_TPL_ID_TRADE1),
        true,
    )
    .await?;
    worker.wait(10).await?;
    check_wf_templates(&worker, &dao_account_id, 2).await?;
    check_instance(
        &worker,
        &dao_account_id,
        proposal_id,
        1,
        InstanceState::Finished,
    )
    .await?;

    // Propose Trade1.
    let proposal_id = proposal_to_finish(
        &worker,
        &member,
        &dao_account_id,
        DAO_TPL_ID_OF_FIRST_ADDED,
        Trade1::propose_settings(
            Some(Trade1ProposeOptions {
                required_token_id: required_token_account_id.to_string(),
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
    storage_deposit(
        &worker,
        &token_holder,
        &other_token_account_id,
        &dao_account_id,
        ONE_NEAR,
    )
    .await?;

    // Transfer tokens to make trade.
    ft_transfer_call(
        &worker,
        &token_holder,
        &other_token_account_id,
        &dao_account_id,
        1_000 * DEFAULT_DECIMALS,
        None,
        serialized_dao_ft_receiver_msg(proposal_id),
    )
    .await?;
    debug_log(&worker, &dao_account_id).await?;
    check_wf_storage_values(&worker, &dao_account_id, "wf_trade1".into(), vec![]).await?;
    let dao_account_balance_before =
        worker.view_account(&dao_account_id).await?.balance / 10u128.pow(24);

    // Execute workflow Trade1.
    run_activity(
        &worker,
        &member,
        &dao_account_id,
        proposal_id,
        1,
        ActivityInputTrade1::activity_1(),
        false,
    )
    .await?;
    worker.wait(5).await?;
    assert_eq!(
        ft_balance_of(&worker, &required_token_account_id, &token_holder.id()).await?,
        (1_000 * DEFAULT_DECIMALS).into()
    );
    assert_eq!(
        ft_balance_of(&worker, &required_token_account_id, &dao_account_id).await?,
        0.into()
    );
    assert_eq!(
        ft_balance_of(&worker, &other_token_account_id, &token_holder.id()).await?,
        0.into()
    );
    assert_eq!(
        ft_balance_of(&worker, &other_token_account_id, &dao_account_id).await?,
        (1_000 * DEFAULT_DECIMALS).into()
    );
    let dao_account_balance_after =
        worker.view_account(&dao_account_id).await?.balance / 10u128.pow(24);
    println!(
        "dao balance before/after: {},{}",
        dao_account_balance_before, dao_account_balance_after
    );
    assert_eq!(dao_account_balance_before, dao_account_balance_after);
    debug_log(&worker, &dao_account_id).await?;
    check_instance(
        &worker,
        &dao_account_id,
        proposal_id,
        0,
        InstanceState::Running,
    )
    .await?;
    Ok(())
}

/// Make bounty and confirm the task was done and send them NEAR as a reward.
/// All values are defined in propose settings.
#[tokio::test]
async fn workflow_bounty1_scenario() -> anyhow::Result<()> {
    let ft_name = "dao_token";
    let dao_name = "test_dao";
    let worker = workspaces::sandbox().await?;
    let member = worker.dev_create_account().await?;
    let bounty_hunter = worker.dev_create_account().await?;

    // Contracts init.
    let ft_factory = init_ft_factory(&worker).await?;
    let factory = init_dao_factory(&worker).await?;
    let dao_account_id = AccountId::try_from(format!("{}.{}", dao_name, factory.id()))
        .expect("invalid dao account id");
    let token_account_id = AccountId::try_from(format!("{}.{}", ft_name, ft_factory.id()))
        .expect("invalid ft account id");
    create_ft_via_factory(
        &worker,
        &ft_factory,
        ft_name,
        dao_account_id.as_str(),
        DAO_FT_TOTAL_SUPPLY,
        24,
        None,
        None,
        vec![],
    )
    .await?;
    let staking = init_staking(&worker).await?;
    let wf_provider = init_workflow_provider(&worker).await?;
    create_dao_via_factory(
        &worker,
        &factory,
        &dao_name,
        &token_account_id,
        DAO_FT_TOTAL_SUPPLY as u32,
        24,
        staking.id(),
        wf_provider.id(),
        factory.id(),
        vec![member.id()],
        0,
    )
    .await?;

    // Storage deposit staking in fungible_token.
    storage_deposit(
        &worker,
        &factory.as_account(),
        &token_account_id,
        staking.id(),
        ONE_NEAR,
    )
    .await?;

    // Load workflows to provider.
    load_workflow_templates(&worker, &wf_provider, None, None).await?;

    let proposal_id = proposal_to_finish(
        &worker,
        &member,
        &dao_account_id,
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
        &dao_account_id,
        proposal_id,
        1,
        ActivityInputWfAdd1::activity_1(wf_provider.id(), PROVIDER_TPL_ID_BOUNTY1),
        true,
    )
    .await?;
    worker.wait(10).await?;
    check_wf_templates(&worker, &dao_account_id, 2).await?;
    check_instance(
        &worker,
        &dao_account_id,
        proposal_id,
        1,
        InstanceState::Finished,
    )
    .await?;

    // Propose Bounty1.
    let proposal_id = proposal_to_finish(
        &worker,
        &member,
        &dao_account_id,
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
        &dao_account_id,
        proposal_id,
        1,
        ActivityInputBounty1::activity_1(),
        true,
    )
    .await?;
    worker.wait(5).await?;
    check_wf_storage_values(&worker, &dao_account_id, "wf_bounty1".into(), vec![]).await?;
    run_activity(
        &worker,
        &member,
        &dao_account_id,
        proposal_id,
        3,
        ActivityInputBounty1::activity_3(true),
        true,
    )
    .await?;
    worker.wait(5).await?;
    check_wf_storage_values(&worker, &dao_account_id, "wf_bounty1".into(), vec![]).await?;
    debug_log(&worker, &dao_account_id).await?;
    run_activity(
        &worker,
        &bounty_hunter,
        &dao_account_id,
        proposal_id,
        4,
        ActivityInputBounty1::activity_4("here link blabla..".into()),
        true,
    )
    .await?;
    worker.wait(5).await?;
    check_wf_storage_values(&worker, &dao_account_id, "wf_bounty1".into(), vec![]).await?;
    run_activity(
        &worker,
        &member,
        &dao_account_id,
        proposal_id,
        5,
        ActivityInputBounty1::activity_5("perfect - 10/10".into()),
        true,
    )
    .await?;
    worker.wait(5).await?;
    let dao_account_balance_before =
        worker.view_account(&dao_account_id).await?.balance / 10u128.pow(24);
    let bounty_hunter_balance_before =
        bounty_hunter.view_account(&worker).await?.balance / 10u128.pow(24);
    check_wf_storage_values(&worker, &dao_account_id, "wf_bounty1".into(), vec![]).await?;
    debug_log(&worker, &dao_account_id).await?;
    run_activity(
        &worker,
        &member,
        &dao_account_id,
        proposal_id,
        6,
        ActivityInputBounty1::activity_6(&bounty_hunter.id(), ONE_NEAR * 10),
        true,
    )
    .await?;
    worker.wait(5).await?;
    let dao_account_balance_after =
        worker.view_account(&dao_account_id).await?.balance / 10u128.pow(24);
    let bounty_hunter_balance_after =
        bounty_hunter.view_account(&worker).await?.balance / 10u128.pow(24);
    assert_eq!(dao_account_balance_before - 10, dao_account_balance_after);
    assert_eq!(
        bounty_hunter_balance_before + 10,
        bounty_hunter_balance_after
    );
    debug_log(&worker, &dao_account_id).await?;
    check_instance(
        &worker,
        &dao_account_id,
        proposal_id,
        6,
        InstanceState::Finished,
    )
    .await?;
    Ok(())
}

/// DAO member adds new partition, new wage reward and then is able to withdraw his reward.
#[tokio::test]
async fn workflow_reward1_wage_scenario() -> anyhow::Result<()> {
    let ft_name = "dao_token";
    let dao_name = "test_dao";
    let reward_token_name = "reward_token";
    let worker = workspaces::sandbox().await?;
    let member = worker.dev_create_account().await?;

    // Contracts init.
    let ft_factory = init_ft_factory(&worker).await?;
    let factory = init_dao_factory(&worker).await?;
    let dao_account_id = AccountId::try_from(format!("{}.{}", dao_name, factory.id()))
        .expect("invalid dao account id");
    let token_account_id = AccountId::try_from(format!("{}.{}", ft_name, ft_factory.id()))
        .expect("invalid ft account id");
    let reward_token_account_id =
        AccountId::try_from(format!("{}.{}", reward_token_name, ft_factory.id()))
            .expect("invalid ft account id");
    create_ft_via_factory(
        &worker,
        &ft_factory,
        ft_name,
        dao_account_id.as_str(),
        DAO_FT_TOTAL_SUPPLY,
        24,
        None,
        None,
        vec![],
    )
    .await?;
    create_ft_via_factory(
        &worker,
        &ft_factory,
        reward_token_name,
        dao_account_id.as_str(),
        DAO_FT_TOTAL_SUPPLY,
        24,
        None,
        None,
        vec![],
    )
    .await?;
    let staking = init_staking(&worker).await?;
    let wf_provider = init_workflow_provider(&worker).await?;
    create_dao_via_factory(
        &worker,
        &factory,
        &dao_name,
        &token_account_id,
        DAO_FT_TOTAL_SUPPLY as u32,
        24,
        staking.id(),
        wf_provider.id(),
        factory.id(),
        vec![member.id()],
        0,
    )
    .await?;
    view_partitions(&worker, &dao_account_id).await?;

    // Storage deposit staking in fungible_token.
    storage_deposit(
        &worker,
        &factory.as_account(),
        &token_account_id,
        staking.id(),
        ONE_NEAR,
    )
    .await?;

    // Load workflows to provider.
    load_workflow_templates(&worker, &wf_provider, None, None).await?;

    let proposal_id = proposal_to_finish(
        &worker,
        &member,
        &dao_account_id,
        DAO_TPL_ID_WF_ADD,
        WfAdd1::propose_settings(Some(WfAdd1ProposeOptions {
            template_id: PROVIDER_TPL_ID_REWARD1,
            provider_id: wf_provider.id().to_string(),
        })),
        Some(vec![Reward1::template_settings(Some(20))]),
        vec![(&member, 1)],
        100,
        WfAdd1::deposit_propose(),
        WfAdd1::deposit_vote(),
        ProposalState::Accepted,
    )
    .await?;

    // Execute AddWorkflow by DAO member to add Reward1 template.
    run_activity(
        &worker,
        &member,
        &dao_account_id,
        proposal_id,
        1,
        ActivityInputWfAdd1::activity_1(wf_provider.id(), PROVIDER_TPL_ID_REWARD1),
        true,
    )
    .await?;
    worker.wait(10).await?;
    check_wf_templates(&worker, &dao_account_id, 2).await?;
    check_instance(
        &worker,
        &dao_account_id,
        proposal_id,
        1,
        InstanceState::Finished,
    )
    .await?;

    // Propose Reward1.
    let proposal_id = proposal_to_finish(
        &worker,
        &member,
        &dao_account_id,
        DAO_TPL_ID_OF_FIRST_ADDED,
        Reward1::propose_settings(Some("wf_reward1")),
        None,
        vec![(&member, 1)],
        100,
        Reward1::deposit_propose(),
        Reward1::deposit_vote(),
        ProposalState::Accepted,
    )
    .await?;
    view_partitions(&worker, &dao_account_id).await?;

    // Execute Workflow Reward1.
    run_activity(
        &worker,
        &member,
        &dao_account_id,
        proposal_id,
        1,
        ActivityInputReward1::activity_1(reward_token_account_id.to_string(), 1000, 24, 1000),
        true,
    )
    .await?;
    worker.wait(5).await?;
    let timestamp = get_timestamp(&worker, &dao_account_id).await?;
    view_partitions(&worker, &dao_account_id).await?;
    run_activity(
        &worker,
        &member,
        &dao_account_id,
        proposal_id,
        2,
        ActivityInputReward1::activity_2(
            10,
            1,
            1,
            3,
            timestamp,
            timestamp + 7200 + 10,
            reward_token_account_id.to_string(),
            3,
            24,
            ONE_NEAR / 8,
        ),
        true,
    )
    .await?;
    worker.wait(5).await?;
    check_instance(
        &worker,
        &dao_account_id,
        proposal_id,
        2,
        InstanceState::Finished,
    )
    .await?;
    debug_log(&worker, &dao_account_id).await?;
    view_user_roles(&worker, &dao_account_id, &member.id()).await?;
    view_user_wallet(&worker, &dao_account_id, &member.id()).await?;
    view_reward(&worker, &dao_account_id, 1).await?;

    // Storage deposit dao in reward token contract so member can withdraw token rewards.
    storage_deposit(
        &worker,
        &member,
        &reward_token_account_id,
        &member.id(),
        ONE_NEAR,
    )
    .await?;
    worker.wait(60).await?;
    ft_balance_of(&worker, &reward_token_account_id, &dao_account_id).await?;
    assert!(
        ft_balance_of(&worker, &reward_token_account_id, &member.id())
            .await?
            .0
            == 0
    );
    let dao_account_balance_before =
        worker.view_account(&dao_account_id).await?.balance / 10u128.pow(24);

    // Withdraw FT reward.
    withdraw_rewards(
        &worker,
        &member,
        &dao_account_id,
        vec![1],
        Asset::new_ft(reward_token_account_id.clone(), 24),
    )
    .await?;
    worker.wait(10).await?;
    assert!(
        ft_balance_of(&worker, &reward_token_account_id, &member.id())
            .await?
            .0
            > 0
    );
    view_user_wallet(&worker, &dao_account_id, &member.id()).await?;
    debug_log(&worker, &dao_account_id).await?;

    // Withdraw NEAR reward.
    withdraw_rewards(
        &worker,
        &member,
        &dao_account_id,
        vec![1],
        Asset::new_near(),
    )
    .await?;

    let dao_account_balance_after =
        worker.view_account(&dao_account_id).await?.balance / 10u128.pow(24);
    assert!(dao_account_balance_before > dao_account_balance_after);
    view_user_wallet(&worker, &dao_account_id, &member.id()).await?;
    view_partitions(&worker, &dao_account_id).await?;
    debug_log(&worker, &dao_account_id).await?;
    Ok(())
}

/// DAO member adds new partition, new wage reward and then is able to withdraw his reward.
#[tokio::test]
async fn workflow_reward1_wage_withdraw_more_near_than_on_dao_account() -> anyhow::Result<()> {
    let ft_name = "dao_token";
    let dao_name = "test_dao";
    let reward_token_name = "reward_token";
    let worker = workspaces::sandbox().await?;
    let member = worker.dev_create_account().await?;

    // Contracts init.
    let ft_factory = init_ft_factory(&worker).await?;
    let factory = init_dao_factory(&worker).await?;
    let dao_account_id = AccountId::try_from(format!("{}.{}", dao_name, factory.id()))
        .expect("invalid dao account id");
    let token_account_id = AccountId::try_from(format!("{}.{}", ft_name, ft_factory.id()))
        .expect("invalid ft account id");
    let reward_token_account_id =
        AccountId::try_from(format!("{}.{}", reward_token_name, ft_factory.id()))
            .expect("invalid ft account id");
    create_ft_via_factory(
        &worker,
        &ft_factory,
        ft_name,
        dao_account_id.as_str(),
        DAO_FT_TOTAL_SUPPLY,
        24,
        None,
        None,
        vec![],
    )
    .await?;
    create_ft_via_factory(
        &worker,
        &ft_factory,
        reward_token_name,
        dao_account_id.as_str(),
        DAO_FT_TOTAL_SUPPLY,
        24,
        None,
        None,
        vec![],
    )
    .await?;
    let staking = init_staking(&worker).await?;
    let wf_provider = init_workflow_provider(&worker).await?;
    create_dao_via_factory(
        &worker,
        &factory,
        &dao_name,
        &token_account_id,
        DAO_FT_TOTAL_SUPPLY as u32,
        24,
        staking.id(),
        wf_provider.id(),
        factory.id(),
        vec![member.id()],
        0,
    )
    .await?;
    view_partitions(&worker, &dao_account_id).await?;

    // Storage deposit staking in fungible_token.
    storage_deposit(
        &worker,
        &factory.as_account(),
        &token_account_id,
        staking.id(),
        ONE_NEAR,
    )
    .await?;

    // Load workflows to provider.
    load_workflow_templates(&worker, &wf_provider, None, None).await?;

    let proposal_id = proposal_to_finish(
        &worker,
        &member,
        &dao_account_id,
        DAO_TPL_ID_WF_ADD,
        WfAdd1::propose_settings(Some(WfAdd1ProposeOptions {
            template_id: PROVIDER_TPL_ID_REWARD1,
            provider_id: wf_provider.id().to_string(),
        })),
        Some(vec![Reward1::template_settings(Some(20))]),
        vec![(&member, 1)],
        100,
        WfAdd1::deposit_propose(),
        WfAdd1::deposit_vote(),
        ProposalState::Accepted,
    )
    .await?;

    // Execute AddWorkflow by DAO member to add Reward1 template.
    run_activity(
        &worker,
        &member,
        &dao_account_id,
        proposal_id,
        1,
        ActivityInputWfAdd1::activity_1(wf_provider.id(), PROVIDER_TPL_ID_REWARD1),
        true,
    )
    .await?;
    worker.wait(10).await?;
    check_wf_templates(&worker, &dao_account_id, 2).await?;
    check_instance(
        &worker,
        &dao_account_id,
        proposal_id,
        1,
        InstanceState::Finished,
    )
    .await?;

    // Propose Reward1.
    let proposal_id = proposal_to_finish(
        &worker,
        &member,
        &dao_account_id,
        DAO_TPL_ID_OF_FIRST_ADDED,
        Reward1::propose_settings(Some("wf_reward1")),
        None,
        vec![(&member, 1)],
        100,
        Reward1::deposit_propose(),
        Reward1::deposit_vote(),
        ProposalState::Accepted,
    )
    .await?;
    view_partitions(&worker, &dao_account_id).await?;

    // Execute Workflow Reward1.
    run_activity(
        &worker,
        &member,
        &dao_account_id,
        proposal_id,
        1,
        ActivityInputReward1::activity_1(reward_token_account_id.to_string(), 1000, 24, 1000),
        true,
    )
    .await?;
    worker.wait(5).await?;
    let timestamp = get_timestamp(&worker, &dao_account_id).await?;
    view_partitions(&worker, &dao_account_id).await?;
    run_activity(
        &worker,
        &member,
        &dao_account_id,
        proposal_id,
        2,
        ActivityInputReward1::activity_2(
            10,
            1,
            1,
            3,
            timestamp,
            timestamp + 7200 + 10,
            reward_token_account_id.to_string(),
            3,
            24,
            10 * ONE_NEAR,
        ),
        true,
    )
    .await?;
    worker.wait(5).await?;
    check_instance(
        &worker,
        &dao_account_id,
        proposal_id,
        2,
        InstanceState::Finished,
    )
    .await?;
    worker.wait(3600).await?;
    debug_log(&worker, &dao_account_id).await?;
    view_partitions(&worker, &dao_account_id).await?;
    statistics(&worker, &dao_account_id).await?;
    view_user_roles(&worker, &dao_account_id, &member.id()).await?;
    view_reward(&worker, &dao_account_id, 1).await?;
    view_user_wallet(&worker, &dao_account_id, &member.id()).await?;

    // Withdraw NEAR reward which exceeds available account balance.
    assert!(withdraw_rewards(
        &worker,
        &member,
        &dao_account_id,
        vec![1],
        Asset::new_near(),
    )
    .await
    .is_err());
    view_partitions(&worker, &dao_account_id).await?;
    statistics(&worker, &dao_account_id).await?;
    view_user_wallet(&worker, &dao_account_id, &member.id()).await?;
    Ok(())
}

/// DAO member adds new partition, new wage reward and then is able to withdraw his reward.
#[tokio::test]
async fn workflow_reward1_user_activity_scenario() -> anyhow::Result<()> {
    let ft_name = "dao_token";
    let dao_name = "test_dao";
    let reward_token_name = "reward_token";
    let worker = workspaces::sandbox().await?;
    let member = worker.dev_create_account().await?;

    // Contracts init.
    let ft_factory = init_ft_factory(&worker).await?;
    let factory = init_dao_factory(&worker).await?;
    let dao_account_id = AccountId::try_from(format!("{}.{}", dao_name, factory.id()))
        .expect("invalid dao account id");
    let token_account_id = AccountId::try_from(format!("{}.{}", ft_name, ft_factory.id()))
        .expect("invalid ft account id");
    let reward_token_account_id =
        AccountId::try_from(format!("{}.{}", reward_token_name, ft_factory.id()))
            .expect("invalid ft account id");
    create_ft_via_factory(
        &worker,
        &ft_factory,
        ft_name,
        dao_account_id.as_str(),
        DAO_FT_TOTAL_SUPPLY,
        24,
        None,
        None,
        vec![],
    )
    .await?;
    create_ft_via_factory(
        &worker,
        &ft_factory,
        reward_token_name,
        dao_account_id.as_str(),
        DAO_FT_TOTAL_SUPPLY,
        24,
        None,
        None,
        vec![],
    )
    .await?;
    let staking = init_staking(&worker).await?;
    let wf_provider = init_workflow_provider(&worker).await?;
    create_dao_via_factory(
        &worker,
        &factory,
        &dao_name,
        &token_account_id,
        DAO_FT_TOTAL_SUPPLY as u32,
        24,
        staking.id(),
        wf_provider.id(),
        factory.id(),
        vec![member.id()],
        0,
    )
    .await?;
    view_partitions(&worker, &dao_account_id).await?;

    // Storage deposit staking in fungible_token.
    storage_deposit(
        &worker,
        &factory.as_account(),
        &token_account_id,
        staking.id(),
        ONE_NEAR,
    )
    .await?;

    // Load workflows to provider.
    load_workflow_templates(&worker, &wf_provider, None, None).await?;

    let proposal_id = proposal_to_finish(
        &worker,
        &member,
        &dao_account_id,
        DAO_TPL_ID_WF_ADD,
        WfAdd1::propose_settings(Some(WfAdd1ProposeOptions {
            template_id: PROVIDER_TPL_ID_REWARD1,
            provider_id: wf_provider.id().to_string(),
        })),
        Some(vec![Reward1::template_settings(Some(20))]),
        vec![(&member, 1)],
        100,
        WfAdd1::deposit_propose(),
        WfAdd1::deposit_vote(),
        ProposalState::Accepted,
    )
    .await?;

    // Execute AddWorkflow by DAO member to add Reward1 template.
    run_activity(
        &worker,
        &member,
        &dao_account_id,
        proposal_id,
        1,
        ActivityInputWfAdd1::activity_1(wf_provider.id(), PROVIDER_TPL_ID_REWARD1),
        true,
    )
    .await?;
    worker.wait(10).await?;
    check_wf_templates(&worker, &dao_account_id, 2).await?;
    check_instance(
        &worker,
        &dao_account_id,
        proposal_id,
        1,
        InstanceState::Finished,
    )
    .await?;

    // Propose Reward1.
    let proposal_id = proposal_to_finish(
        &worker,
        &member,
        &dao_account_id,
        DAO_TPL_ID_OF_FIRST_ADDED,
        Reward1::propose_settings(Some("wf_reward1")),
        None,
        vec![(&member, 1)],
        100,
        Reward1::deposit_propose(),
        Reward1::deposit_vote(),
        ProposalState::Accepted,
    )
    .await?;

    // Execute Workflow Reward1.
    run_activity(
        &worker,
        &member,
        &dao_account_id,
        proposal_id,
        1,
        ActivityInputReward1::activity_1(reward_token_account_id.to_string(), 1000, 24, 1000),
        true,
    )
    .await?;
    worker.wait(5).await?;
    let timestamp = get_timestamp(&worker, &dao_account_id).await?;
    run_activity(
        &worker,
        &member,
        &dao_account_id,
        proposal_id,
        3,
        ActivityInputReward1::activity_3(
            vec![RewardActivity::AcceptedProposal, RewardActivity::Vote],
            1,
            1,
            3,
            timestamp,
            timestamp + 7200 + 10,
            reward_token_account_id.to_string(),
            1 * ONE_NEAR,
            24,
            1 * ONE_NEAR,
        ),
        true,
    )
    .await?;
    worker.wait(5).await?;
    check_instance(
        &worker,
        &dao_account_id,
        proposal_id,
        3,
        InstanceState::Finished,
    )
    .await?;
    debug_log(&worker, &dao_account_id).await?;
    view_user_roles(&worker, &dao_account_id, &member.id()).await?;
    view_user_wallet(&worker, &dao_account_id, &member.id()).await?;
    view_reward(&worker, &dao_account_id, 1).await?;

    // Storage deposit dao in reward token contract so member can withdraw token rewards.
    storage_deposit(
        &worker,
        &member,
        &reward_token_account_id,
        &member.id(),
        ONE_NEAR,
    )
    .await?;
    worker.wait(3600).await?;
    ft_balance_of(&worker, &reward_token_account_id, &dao_account_id).await?;
    assert!(
        ft_balance_of(&worker, &reward_token_account_id, &member.id())
            .await?
            .0
            == 0
    );
    let dao_account_balance_before =
        worker.view_account(&dao_account_id).await?.balance / 10u128.pow(24);

    // Withdraw FT reward.
    withdraw_rewards(
        &worker,
        &member,
        &dao_account_id,
        vec![1],
        Asset::new_ft(reward_token_account_id.clone(), 24),
    )
    .await?;
    worker.wait(10).await?;
    debug_log(&worker, &dao_account_id).await?;
    view_user_wallet(&worker, &dao_account_id, &member.id()).await?;
    assert!(
        ft_balance_of(&worker, &reward_token_account_id, &member.id())
            .await?
            .0
            == 0
    );

    // Withdraw NEAR reward.
    withdraw_rewards(
        &worker,
        &member,
        &dao_account_id,
        vec![1],
        Asset::new_near(),
    )
    .await?;
    let dao_account_balance_after =
        worker.view_account(&dao_account_id).await?.balance / 10u128.pow(24);
    debug_log(&worker, &dao_account_id).await?;
    view_user_wallet(&worker, &dao_account_id, &member.id()).await?;
    assert!(dao_account_balance_before == dao_account_balance_after);
    view_partitions(&worker, &dao_account_id).await?;

    // Propose any proposal to test activity rewarding.
    let proposal_id = proposal_to_finish(
        &worker,
        &member,
        &dao_account_id,
        DAO_TPL_ID_OF_FIRST_ADDED,
        Reward1::propose_settings(Some("wf_reward2")),
        None,
        vec![(&member, 1)],
        100,
        Reward1::deposit_propose(),
        Reward1::deposit_vote(),
        ProposalState::Accepted,
    )
    .await?;
    worker.wait(5).await?;

    // Check generated rewards for user's activity.
    check_instance(
        &worker,
        &dao_account_id,
        proposal_id,
        0,
        InstanceState::Running,
    )
    .await?;
    debug_log(&worker, &dao_account_id).await?;
    view_user_wallet(&worker, &dao_account_id, &member.id()).await?;
    view_reward(&worker, &dao_account_id, 1).await?;
    ft_balance_of(&worker, &reward_token_account_id, &dao_account_id).await?;
    assert!(
        ft_balance_of(&worker, &reward_token_account_id, &member.id())
            .await?
            .0
            == 0
    );
    let dao_account_balance_before =
        worker.view_account(&dao_account_id).await?.balance / 10u128.pow(24);

    // Withdraw FT reward.
    withdraw_rewards(
        &worker,
        &member,
        &dao_account_id,
        vec![1],
        Asset::new_ft(reward_token_account_id.clone(), 24),
    )
    .await?;
    worker.wait(10).await?;
    debug_log(&worker, &dao_account_id).await?;
    view_user_wallet(&worker, &dao_account_id, &member.id()).await?;
    assert_eq!(
        ft_balance_of(&worker, &reward_token_account_id, &member.id())
            .await?
            .0,
        2 * ONE_NEAR
    );

    // Withdraw NEAR reward.
    withdraw_rewards(
        &worker,
        &member,
        &dao_account_id,
        vec![1],
        Asset::new_near(),
    )
    .await?;
    let dao_account_balance_after =
        worker.view_account(&dao_account_id).await?.balance / 10u128.pow(24);
    debug_log(&worker, &dao_account_id).await?;
    view_user_wallet(&worker, &dao_account_id, &member.id()).await?;
    assert_eq!(dao_account_balance_before, dao_account_balance_after + 2);
    view_partitions(&worker, &dao_account_id).await?;
    Ok(())
}

#[tokio::test]
async fn workflow_admin_package() -> anyhow::Result<()> {
    let ft_name = "dao_token";
    let dao_name = "test_dao";
    let worker = workspaces::sandbox().await?;
    let member = worker.dev_create_account().await?;

    // Contracts init.
    let ft_factory = init_ft_factory(&worker).await?;
    let factory = init_dao_factory(&worker).await?;
    let dao_account_id = AccountId::try_from(format!("{}.{}", dao_name, factory.id()))
        .expect("invalid dao account id");
    let token_account_id = AccountId::try_from(format!("{}.{}", ft_name, ft_factory.id()))
        .expect("invalid ft account id");

    create_ft_via_factory(
        &worker,
        &ft_factory,
        ft_name,
        dao_account_id.as_str(),
        DAO_FT_TOTAL_SUPPLY,
        24,
        None,
        None,
        vec![],
    )
    .await?;
    let staking = init_staking(&worker).await?;
    let wf_provider = init_workflow_provider(&worker).await?;
    create_dao_via_factory(
        &worker,
        &factory,
        &dao_name,
        &token_account_id,
        DAO_FT_TOTAL_SUPPLY as u32,
        24,
        staking.id(),
        wf_provider.id(),
        factory.id(),
        vec![member.id()],
        0,
    )
    .await?;
    view_partitions(&worker, &dao_account_id).await?;

    // Storage deposit staking in fungible_token.
    storage_deposit(
        &worker,
        &factory.as_account(),
        &token_account_id,
        staking.id(),
        ONE_NEAR,
    )
    .await?;

    // Load workflows to provider.
    load_workflow_templates(&worker, &wf_provider, None, None).await?;

    let proposal_id = proposal_to_finish(
        &worker,
        &member,
        &dao_account_id,
        DAO_TPL_ID_WF_ADD,
        WfAdd1::propose_settings(Some(WfAdd1ProposeOptions {
            template_id: PROVIDER_TPL_ID_ADMIN_PACKAGE1,
            provider_id: wf_provider.id().to_string(),
        })),
        Some(vec![AdminPackage1::template_settings(Some(20))]),
        vec![(&member, 1)],
        100,
        WfAdd1::deposit_propose(),
        WfAdd1::deposit_vote(),
        ProposalState::Accepted,
    )
    .await?;

    // Execute AddWorkflow by DAO member to add AdminPackage1 template.
    run_activity(
        &worker,
        &member,
        &dao_account_id,
        proposal_id,
        1,
        ActivityInputWfAdd1::activity_1(wf_provider.id(), PROVIDER_TPL_ID_ADMIN_PACKAGE1),
        true,
    )
    .await?;
    worker.wait(10).await?;
    check_wf_templates(&worker, &dao_account_id, 2).await?;
    check_instance(
        &worker,
        &dao_account_id,
        proposal_id,
        1,
        InstanceState::Finished,
    )
    .await?;

    // Propose AdminPackage1.
    let proposal_id = proposal_to_finish(
        &worker,
        &member,
        &dao_account_id,
        DAO_TPL_ID_OF_FIRST_ADDED,
        AdminPackage1::propose_settings(),
        None,
        vec![(&member, 1)],
        100,
        AdminPackage1::deposit_propose(),
        AdminPackage1::deposit_vote(),
        ProposalState::Accepted,
    )
    .await?;
    view_partitions(&worker, &dao_account_id).await?;

    // Execute Workflow AdminPackage1.
    // GroupAdd - artists
    run_activity(
        &worker,
        &member,
        &dao_account_id,
        proposal_id,
        ADMINPACKAGE1_ADD_GROUP,
        ActivityInputAdminPkg1::activity_group_add(
            "artists",
            "macho.near",
            vec!["macho.near", "pica.near"],
            Some("alpha"),
            vec!["macho.near"],
        ),
        true,
    )
    .await?;
    worker.wait(5).await?;
    check_group(
        &worker,
        &dao_account_id,
        2,
        "artists",
        Some("macho.near"),
        0,
        vec![("macho.near", vec![]), ("pica.near", vec![])],
        vec![],
    )
    .await?;
    let macho_roles = UserRoles::new().add_group_roles(2, vec![0, 1]);
    let pica_roles = UserRoles::new().add_group_roles(2, vec![0]);
    check_group_exists(&worker, &dao_account_id, "artists", true).await?;
    check_user_roles(
        &worker,
        &dao_account_id,
        "macho.near",
        Some(&macho_roles.clone()),
    )
    .await?;
    check_user_roles(
        &worker,
        &dao_account_id,
        "pica.near",
        Some(&pica_roles.clone()),
    )
    .await?;
    let artists_roles = Roles::new().add_role("alpha");
    check_group_roles(&worker, &dao_account_id, 2, &artists_roles).await?;

    // GroupAddMembers (and roles)
    run_activity(
        &worker,
        &member,
        &dao_account_id,
        proposal_id,
        ADMINPACKAGE1_ADD_GROUP_MEMBERS,
        ActivityInputAdminPkg1::activity_group_add_members(
            2,
            vec!["abc.near", "def.near"],
            vec![
                ("some_role", vec!["no_one_gets_this_role.near"]),
                ("alpha", vec!["abc.near"]),
                ("omega", vec!["pica.near"]),
            ],
        ),
        true,
    )
    .await?;
    check_group(
        &worker,
        &dao_account_id,
        2,
        "artists",
        Some("macho.near"),
        0,
        vec![
            ("macho.near", vec![]),
            ("pica.near", vec![]),
            ("abc.near", vec![]),
            ("def.near", vec![]),
        ],
        vec![],
    )
    .await?;
    let abc_roles = macho_roles.clone();
    let def_roles = pica_roles.clone();
    check_user_roles(&worker, &dao_account_id, "macho.near", Some(&macho_roles)).await?;
    check_user_roles(
        &worker,
        &dao_account_id,
        "pica.near",
        Some(&pica_roles.clone().add_role(2, 3)),
    )
    .await?;
    check_user_roles(&worker, &dao_account_id, "abc.near", Some(&abc_roles)).await?;
    check_user_roles(&worker, &dao_account_id, "def.near", Some(&def_roles)).await?;
    let artists_roles = artists_roles.add_role("some_role").add_role("omega");
    check_group_roles(&worker, &dao_account_id, 2, &artists_roles).await?;

    // GroupRemoveMember
    run_activity(
        &worker,
        &member,
        &dao_account_id,
        proposal_id,
        ADMINPACKAGE1_REMOVE_GROUP_MEMBERS,
        ActivityInputAdminPkg1::activity_group_remove_members(2, vec!["def.near"]),
        true,
    )
    .await?;
    check_group(
        &worker,
        &dao_account_id,
        2,
        "artists",
        Some("macho.near"),
        0,
        vec![
            ("macho.near", vec![]),
            ("pica.near", vec![]),
            ("abc.near", vec![]),
        ],
        vec![],
    )
    .await?;
    check_user_roles(&worker, &dao_account_id, "macho.near", Some(&macho_roles)).await?;
    check_user_roles(
        &worker,
        &dao_account_id,
        "pica.near",
        Some(&pica_roles.add_role(2, 3)),
    )
    .await?;
    check_user_roles(&worker, &dao_account_id, "abc.near", Some(&abc_roles)).await?;
    check_user_roles(&worker, &dao_account_id, "def.near", None).await?;

    // GroupRemoveRole - alpha
    let artists_roles = artists_roles.remove_role("alpha");
    run_activity(
        &worker,
        &member,
        &dao_account_id,
        proposal_id,
        ADMINPACKAGE1_REMOVE_GROUP_ROLES,
        ActivityInputAdminPkg1::activity_group_remove_roles(2, vec![1, 4, 5, 6]),
        true,
    )
    .await?;
    check_group_roles(&worker, &dao_account_id, 2, &artists_roles.clone()).await?;
    let expected_role = UserRoles::new().add_role(2, 0);
    check_user_roles(
        &worker,
        &dao_account_id,
        "macho.near",
        Some(&expected_role.clone()),
    )
    .await?;
    check_user_roles(
        &worker,
        &dao_account_id,
        "pica.near",
        Some(&expected_role.clone().add_role(2, 3)),
    )
    .await?;
    check_user_roles(
        &worker,
        &dao_account_id,
        "abc.near",
        Some(&expected_role.clone()),
    )
    .await?;
    check_user_roles(&worker, &dao_account_id, "def.near", None).await?;

    // GroupRemoveMemberRoles - gamma
    run_activity(
        &worker,
        &member,
        &dao_account_id,
        proposal_id,
        ADMINPACKAGE1_REMOVE_GROUP_MEMBER_ROLES,
        ActivityInputAdminPkg1::activity_group_remove_member_roles(
            2,
            vec![("omega", vec![]), ("some_role", vec![])],
        ),
        true,
    )
    .await?;
    check_group_roles(&worker, &dao_account_id, 2, &artists_roles).await?;
    let expected_role = UserRoles::new().add_role(2, 0);
    check_user_roles(
        &worker,
        &dao_account_id,
        "macho.near",
        Some(&expected_role.clone()),
    )
    .await?;
    check_user_roles(
        &worker,
        &dao_account_id,
        "pica.near",
        Some(&expected_role.clone().add_role(2, 3)),
    )
    .await?;
    check_user_roles(
        &worker,
        &dao_account_id,
        "abc.near",
        Some(&expected_role.clone()),
    )
    .await?;
    check_user_roles(&worker, &dao_account_id, "def.near", None).await?;

    // GroupRemove - artists
    run_activity(
        &worker,
        &member,
        &dao_account_id,
        proposal_id,
        ADMINPACKAGE1_REMOVE_GROUP,
        ActivityInputAdminPkg1::activity_group_remove(2),
        true,
    )
    .await?;
    view_groups(&worker, &dao_account_id).await?;
    check_user_roles(&worker, &dao_account_id, "macho.near", None).await?;
    check_user_roles(&worker, &dao_account_id, "pica.near", None).await?;
    check_user_roles(&worker, &dao_account_id, "abc.near", None).await?;
    check_group_exists(&worker, &dao_account_id, "artists", false).await?;

    Ok(())
}
