use crate::constants::{
    DAO_FT_TOTAL_SUPPLY, DAO_TPL_ID_OF_FIRST_ADDED, DAO_TPL_ID_WF_ADD, PROVIDER_TPL_ID_BOUNTY1,
};
use crate::types::ProposalState;
use crate::utils::{
    check_instance, check_wf_storage_values, check_wf_templates, create_dao_via_factory,
    create_ft_via_factory, debug_log, init_dao_factory, init_ft_factory, init_staking,
    init_workflow_provider, load_workflow_templates, proposal_to_finish, run_activity,
    storage_deposit, ActivityInputBounty1, ActivityInputWfAdd1, Wait,
};
use near_sdk::ONE_NEAR;

use data::workflow::basic::bounty::{Bounty1, Bounty1ProposeOptions};
use data::workflow::basic::wf_add::{WfAdd1, WfAdd1ProposeOptions};
use library::workflow::instance::InstanceState;
use workspaces::{network::DevAccountDeployer, AccountId};

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
        1,
        InstanceState::Finished,
    )
    .await?;
    Ok(())
}
