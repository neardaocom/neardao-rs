use near_sdk::ONE_NEAR;

use crate::constants::{
    DAO_TPL_ID_OF_FIRST_ADDED, DAO_TPL_ID_WF_ADD, PROVIDER_TPL_ID_TEST_OPTIONAL_ACTIONS,
};
use crate::test_data::WfOptionalActions;
use crate::types::ProposalState;
use crate::utils::{
    check_instance, check_wf_storage_values, check_wf_templates, create_dao_via_factory,
    create_ft_via_factory, debug_log, init_dao_factory, init_ft_factory, init_skyward,
    init_staking, init_wnear, init_workflow_provider, load_workflow_templates, proposal_to_finish,
    run_activity, statistics, storage_deposit, wf_log, workflow_finish, workflow_storage_buckets,
    ActivityInputTestOptionalActions as ActivityInput, ActivityInputWfAdd1, Wait,
};

use data::workflow::basic::wf_add::{WfAdd1, WfAdd1ProposeOptions};
use library::{types::datatype::Value, workflow::instance::InstanceState};
use workspaces::{network::DevAccountDeployer, AccountId};

const DAO_FT_TOTAL_SUPPLY: u128 = 1_000_000_000;
const DEFAULT_DECIMALS: u128 = 10u128.pow(24);

/// To test optional actions are working correctly.
#[tokio::test]
async fn test_workflow_optional_actions() -> anyhow::Result<()> {
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
            template_id: PROVIDER_TPL_ID_TEST_OPTIONAL_ACTIONS,
            provider_id: wf_provider.id().to_string(),
        })),
        Some(vec![WfOptionalActions::template_settings(None)]),
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
        ActivityInputWfAdd1::activity_1(wf_provider.id(), PROVIDER_TPL_ID_TEST_OPTIONAL_ACTIONS),
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

    // Propose WfOptionalActions.
    let proposal_id = proposal_to_finish(
        &worker,
        &member,
        &dao_account_id,
        DAO_TPL_ID_OF_FIRST_ADDED,
        WfOptionalActions::propose_settings("first"),
        None,
        vec![(&member, 1)],
        100,
        WfOptionalActions::deposit_propose(),
        WfOptionalActions::deposit_vote(),
        ProposalState::Accepted,
    )
    .await?;

    // Execute workflow first activity all.
    run_activity(
        &worker,
        &member,
        &dao_account_id,
        proposal_id,
        1,
        ActivityInput::activity_1_complete(),
        true,
    )
    .await?;
    worker.wait(5).await?;
    check_instance(
        &worker,
        &dao_account_id,
        proposal_id,
        1,
        4,
        InstanceState::Running,
    )
    .await?;
    workflow_storage_buckets(&worker, &dao_account_id).await?;
    check_wf_storage_values(
        &worker,
        &dao_account_id,
        "global".into(),
        vec![
            ("pp_activity_1_action_0".into(), Value::Bool(true)),
            ("pp_activity_1_action_1".into(), Value::Bool(true)),
            ("pp_activity_1_action_2".into(), Value::Bool(true)),
            ("pp_activity_1_action_3".into(), Value::Bool(true)),
        ],
    )
    .await?;
    let member_balance_before = worker.view_account(member.id()).await?.balance / 10u128.pow(24);

    // Execute workflow second activity all. - Only first actions should be executed as 2th and 3rd are optional.
    run_activity(
        &worker,
        &member,
        &dao_account_id,
        proposal_id,
        2,
        ActivityInput::activity_2_complete_optional_missing(),
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
        InstanceState::Running,
    )
    .await?;
    workflow_storage_buckets(&worker, &dao_account_id).await?;
    check_wf_storage_values(
        &worker,
        &dao_account_id,
        "global".into(),
        vec![
            ("pp_activity_1_action_0".into(), Value::Bool(true)),
            ("pp_activity_1_action_1".into(), Value::Bool(true)),
            ("pp_activity_1_action_2".into(), Value::Bool(true)),
            ("pp_activity_1_action_3".into(), Value::Bool(true)),
            ("pp_activity_2_action_0".into(), Value::Bool(true)),
        ],
    )
    .await?;

    // Execute workflow second activity - 4th action, skipping 2th and 3rd actions.
    run_activity(
        &worker,
        &member,
        &dao_account_id,
        proposal_id,
        2,
        ActivityInput::activity_2_complete_rest(),
        true,
    )
    .await?;
    worker.wait(5).await?;
    check_instance(
        &worker,
        &dao_account_id,
        proposal_id,
        2,
        4,
        InstanceState::Running,
    )
    .await?;
    workflow_storage_buckets(&worker, &dao_account_id).await?;
    check_wf_storage_values(
        &worker,
        &dao_account_id,
        "global".into(),
        vec![
            ("pp_activity_1_action_0".into(), Value::Bool(true)),
            ("pp_activity_1_action_1".into(), Value::Bool(true)),
            ("pp_activity_1_action_2".into(), Value::Bool(true)),
            ("pp_activity_1_action_3".into(), Value::Bool(true)),
            ("pp_activity_2_action_0".into(), Value::Bool(true)),
            ("pp_activity_2_action_3".into(), Value::Bool(true)),
        ],
    )
    .await?;
    debug_log(&worker, &dao_account_id).await?;
    statistics(&worker, &dao_account_id).await?;
    wf_log(&worker, &dao_account_id, proposal_id).await?;
    let member_balance_after = worker.view_account(member.id()).await?.balance / 10u128.pow(24);
    assert!(member_balance_before + 10 == member_balance_after);
    workflow_finish(&worker, &member, &dao_account_id, proposal_id, true).await?;

    Ok(())
}

#[tokio::test]
async fn test_workflow_optional_actions_cannot_leave_activity() -> anyhow::Result<()> {
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
            template_id: PROVIDER_TPL_ID_TEST_OPTIONAL_ACTIONS,
            provider_id: wf_provider.id().to_string(),
        })),
        Some(vec![WfOptionalActions::template_settings(None)]),
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
        ActivityInputWfAdd1::activity_1(wf_provider.id(), PROVIDER_TPL_ID_TEST_OPTIONAL_ACTIONS),
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

    // Propose WfOptionalActions.
    let proposal_id = proposal_to_finish(
        &worker,
        &member,
        &dao_account_id,
        DAO_TPL_ID_OF_FIRST_ADDED,
        WfOptionalActions::propose_settings("first"),
        None,
        vec![(&member, 1)],
        100,
        WfOptionalActions::deposit_propose(),
        WfOptionalActions::deposit_vote(),
        ProposalState::Accepted,
    )
    .await?;

    // Execute workflow first activity 1th action.
    run_activity(
        &worker,
        &member,
        &dao_account_id,
        proposal_id,
        1,
        ActivityInput::activity_1_action_0(),
        true,
    )
    .await?;
    worker.wait(5).await?;
    check_instance(
        &worker,
        &dao_account_id,
        proposal_id,
        1,
        1,
        InstanceState::Running,
    )
    .await?;
    workflow_storage_buckets(&worker, &dao_account_id).await?;
    check_wf_storage_values(
        &worker,
        &dao_account_id,
        "global".into(),
        vec![("pp_activity_1_action_0".into(), Value::Bool(true))],
    )
    .await?;

    // Cannot skip to 2. when 1. is in progress.
    run_activity(
        &worker,
        &member,
        &dao_account_id,
        proposal_id,
        2,
        ActivityInput::activity_2_complete_optional_missing(),
        false,
    )
    .await?;
    worker.wait(5).await?;
    check_instance(
        &worker,
        &dao_account_id,
        proposal_id,
        1,
        1,
        InstanceState::Running,
    )
    .await?;
    workflow_storage_buckets(&worker, &dao_account_id).await?;
    check_wf_storage_values(
        &worker,
        &dao_account_id,
        "global".into(),
        vec![("pp_activity_1_action_0".into(), Value::Bool(true))],
    )
    .await?;

    // Finish activity 1 by skipping 2th and 3rd action.
    run_activity(
        &worker,
        &member,
        &dao_account_id,
        proposal_id,
        1,
        ActivityInput::activity_1_action_3_skip_2(),
        true,
    )
    .await?;
    worker.wait(5).await?;
    check_instance(
        &worker,
        &dao_account_id,
        proposal_id,
        1,
        4,
        InstanceState::Running,
    )
    .await?;
    workflow_storage_buckets(&worker, &dao_account_id).await?;
    check_wf_storage_values(
        &worker,
        &dao_account_id,
        "global".into(),
        vec![
            ("pp_activity_1_action_0".into(), Value::Bool(true)),
            ("pp_activity_1_action_3".into(), Value::Bool(true)),
        ],
    )
    .await?;

    // Finish activity 2 in one call.
    let member_balance_before = worker.view_account(member.id()).await?.balance / 10u128.pow(24);
    run_activity(
        &worker,
        &member,
        &dao_account_id,
        proposal_id,
        2,
        ActivityInput::activity_2_complete(),
        true,
    )
    .await?;
    worker.wait(5).await?;
    check_instance(
        &worker,
        &dao_account_id,
        proposal_id,
        2,
        4,
        InstanceState::Running,
    )
    .await?;
    workflow_storage_buckets(&worker, &dao_account_id).await?;
    check_wf_storage_values(
        &worker,
        &dao_account_id,
        "global".into(),
        vec![
            ("pp_activity_1_action_0".into(), Value::Bool(true)),
            ("pp_activity_1_action_3".into(), Value::Bool(true)),
            ("pp_activity_2_action_0".into(), Value::Bool(true)),
            ("pp_activity_2_action_1".into(), Value::Bool(true)),
            ("pp_activity_2_action_2".into(), Value::Bool(true)),
            ("pp_activity_2_action_3".into(), Value::Bool(true)),
        ],
    )
    .await?;

    debug_log(&worker, &dao_account_id).await?;
    statistics(&worker, &dao_account_id).await?;
    let member_balance_after = worker.view_account(member.id()).await?.balance / 10u128.pow(24);
    assert!(member_balance_before + 20 == member_balance_after);
    workflow_finish(&worker, &member, &dao_account_id, proposal_id, true).await?;

    Ok(())
}

#[tokio::test]
async fn test_workflow_optional_actions_execute_activity_1_without_3rd_action() -> anyhow::Result<()>
{
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
            template_id: PROVIDER_TPL_ID_TEST_OPTIONAL_ACTIONS,
            provider_id: wf_provider.id().to_string(),
        })),
        Some(vec![WfOptionalActions::template_settings(None)]),
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
        ActivityInputWfAdd1::activity_1(wf_provider.id(), PROVIDER_TPL_ID_TEST_OPTIONAL_ACTIONS),
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

    // Propose WfOptionalActions.
    let proposal_id = proposal_to_finish(
        &worker,
        &member,
        &dao_account_id,
        DAO_TPL_ID_OF_FIRST_ADDED,
        WfOptionalActions::propose_settings("first"),
        None,
        vec![(&member, 1)],
        100,
        WfOptionalActions::deposit_propose(),
        WfOptionalActions::deposit_vote(),
        ProposalState::Accepted,
    )
    .await?;

    // Execute workflow first activity 1th action.
    run_activity(
        &worker,
        &member,
        &dao_account_id,
        proposal_id,
        1,
        ActivityInput::activity_1_action_0(),
        true,
    )
    .await?;
    worker.wait(5).await?;
    check_instance(
        &worker,
        &dao_account_id,
        proposal_id,
        1,
        1,
        InstanceState::Running,
    )
    .await?;
    workflow_storage_buckets(&worker, &dao_account_id).await?;
    check_wf_storage_values(
        &worker,
        &dao_account_id,
        "global".into(),
        vec![("pp_activity_1_action_0".into(), Value::Bool(true))],
    )
    .await?;

    // Execute workflow first activity 2th action.
    run_activity(
        &worker,
        &member,
        &dao_account_id,
        proposal_id,
        1,
        ActivityInput::activity_1_action_1(),
        true,
    )
    .await?;
    worker.wait(5).await?;
    check_instance(
        &worker,
        &dao_account_id,
        proposal_id,
        1,
        2,
        InstanceState::Running,
    )
    .await?;
    workflow_storage_buckets(&worker, &dao_account_id).await?;
    check_wf_storage_values(
        &worker,
        &dao_account_id,
        "global".into(),
        vec![
            ("pp_activity_1_action_0".into(), Value::Bool(true)),
            ("pp_activity_1_action_1".into(), Value::Bool(true)),
        ],
    )
    .await?;

    // Finish activity 1 by skipping 3rd action.
    run_activity(
        &worker,
        &member,
        &dao_account_id,
        proposal_id,
        1,
        ActivityInput::activity_1_action_3_skip_previous(),
        true,
    )
    .await?;
    worker.wait(5).await?;
    check_instance(
        &worker,
        &dao_account_id,
        proposal_id,
        1,
        4,
        InstanceState::Running,
    )
    .await?;
    workflow_storage_buckets(&worker, &dao_account_id).await?;
    check_wf_storage_values(
        &worker,
        &dao_account_id,
        "global".into(),
        vec![
            ("pp_activity_1_action_0".into(), Value::Bool(true)),
            ("pp_activity_1_action_1".into(), Value::Bool(true)),
            ("pp_activity_1_action_3".into(), Value::Bool(true)),
        ],
    )
    .await?;
    debug_log(&worker, &dao_account_id).await?;
    statistics(&worker, &dao_account_id).await?;
    workflow_finish(&worker, &member, &dao_account_id, proposal_id, false).await?;

    Ok(())
}
