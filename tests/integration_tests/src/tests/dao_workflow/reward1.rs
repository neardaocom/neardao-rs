use near_sdk::ONE_NEAR;

use crate::constants::{DAO_TPL_ID_OF_FIRST_ADDED, DAO_TPL_ID_WF_ADD, PROVIDER_TPL_ID_REWARD1};
use crate::types::{Asset, ProposalState, RewardActivity};
use crate::utils::{
    check_instance, check_wf_templates, create_dao_via_factory, create_ft_via_factory, debug_log,
    ft_balance_of, get_timestamp, init_dao_factory, init_ft_factory, init_staking,
    init_workflow_provider, load_workflow_templates, proposal_to_finish, run_activity, statistics,
    storage_deposit, view_partitions, view_reward, view_user_roles, view_user_wallet,
    withdraw_rewards, ActivityInputReward1, ActivityInputWfBasicPkg1, Wait,
};

use data::workflow::basic::{
    basic_package::{WfBasicPkg1, WfBasicPkg1ProposeOptions},
    reward::Reward1,
};
use library::workflow::instance::InstanceState;
use workspaces::{network::DevAccountDeployer, AccountId};

const DAO_FT_TOTAL_SUPPLY: u128 = 1_000_000_000;
const DEFAULT_DECIMALS: u128 = 10u128.pow(24);

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
        WfBasicPkg1::propose_settings(Some(WfBasicPkg1ProposeOptions {
            template_id: PROVIDER_TPL_ID_REWARD1,
            provider_id: wf_provider.id().to_string(),
        })),
        Some(vec![Reward1::template_settings(Some(20))]),
        vec![(&member, 1)],
        100,
        WfBasicPkg1::deposit_propose(),
        WfBasicPkg1::deposit_vote(),
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
        ActivityInputWfBasicPkg1::activity_1(wf_provider.id(), PROVIDER_TPL_ID_REWARD1),
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
        1,
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
        WfBasicPkg1::propose_settings(Some(WfBasicPkg1ProposeOptions {
            template_id: PROVIDER_TPL_ID_REWARD1,
            provider_id: wf_provider.id().to_string(),
        })),
        Some(vec![Reward1::template_settings(Some(20))]),
        vec![(&member, 1)],
        100,
        WfBasicPkg1::deposit_propose(),
        WfBasicPkg1::deposit_vote(),
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
        ActivityInputWfBasicPkg1::activity_1(wf_provider.id(), PROVIDER_TPL_ID_REWARD1),
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
        1,
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

/// DAO member adds new partition, new user activity reward and then is able to withdraw his reward.
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
        WfBasicPkg1::propose_settings(Some(WfBasicPkg1ProposeOptions {
            template_id: PROVIDER_TPL_ID_REWARD1,
            provider_id: wf_provider.id().to_string(),
        })),
        Some(vec![Reward1::template_settings(Some(20))]),
        vec![(&member, 1)],
        100,
        WfBasicPkg1::deposit_propose(),
        WfBasicPkg1::deposit_vote(),
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
        ActivityInputWfBasicPkg1::activity_1(wf_provider.id(), PROVIDER_TPL_ID_REWARD1),
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
        1,
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
async fn workflow_reward1_skip_partition_creation_wage_scenario() -> anyhow::Result<()> {
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
        WfBasicPkg1::propose_settings(Some(WfBasicPkg1ProposeOptions {
            template_id: PROVIDER_TPL_ID_REWARD1,
            provider_id: wf_provider.id().to_string(),
        })),
        Some(vec![Reward1::template_settings(Some(20))]),
        vec![(&member, 1)],
        100,
        WfBasicPkg1::deposit_propose(),
        WfBasicPkg1::deposit_vote(),
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
        ActivityInputWfBasicPkg1::activity_1(wf_provider.id(), PROVIDER_TPL_ID_REWARD1),
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
    let timestamp = get_timestamp(&worker, &dao_account_id).await?;
    view_partitions(&worker, &dao_account_id).await?;
    run_activity(
        &worker,
        &member,
        &dao_account_id,
        proposal_id,
        2,
        ActivityInputReward1::activity_2_only_near_wage(
            10,
            1,
            1,
            1,
            timestamp,
            timestamp + 7200 + 10,
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
        1,
        InstanceState::Finished,
    )
    .await?;
    debug_log(&worker, &dao_account_id).await?;
    view_user_roles(&worker, &dao_account_id, &member.id()).await?;
    view_user_wallet(&worker, &dao_account_id, &member.id()).await?;
    view_reward(&worker, &dao_account_id, 1).await?;
    let dao_account_balance_before =
        worker.view_account(&dao_account_id).await?.balance / 10u128.pow(24);
    worker.wait(10).await?;

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
