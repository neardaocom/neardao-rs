use std::collections::HashMap;

use data::workflow::basic::basic_package::{WfBasicPkg1, WfBasicPkg1ProposeOptions};
use data::workflow::basic::lock::Lock1;
use near_sdk::ONE_NEAR;

use crate::constants::{DAO_TPL_ID_OF_FIRST_ADDED, DAO_TPL_ID_WF_ADD, PROVIDER_TPL_ID_LOCK1};
#[allow(unused_imports)]
use crate::utils::{
    check_group_roles, check_instance, check_sale, check_user_roles, check_wf_storage_values,
    check_wf_templates, create_dao_via_factory, create_ft_via_factory, debug_log, ft_balance_of,
    init_dao_factory, init_ft_factory, init_skyward, init_staking, init_wnear,
    init_workflow_provider, load_workflow_templates, proposal_to_finish,
    proposal_to_finish_testnet, run_activity, statistics, storage_deposit, wf_log,
    ActivityInputWfBasicPkg1, Wait,
};
use crate::{
    types::ProposalState,
    utils::{check_partitions, ActivityInputLock1},
};
use library::workflow::instance::InstanceState;
use workspaces::{network::DevAccountDeployer, AccountId};

const DAO_FT_TOTAL_SUPPLY: u128 = 1_000_000_000;
const DEFAULT_DECIMALS: u128 = 10u128.pow(24);

#[tokio::test]
async fn workflow_lock1_scenario() -> anyhow::Result<()> {
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
        WfBasicPkg1::propose_settings(Some(WfBasicPkg1ProposeOptions {
            template_id: PROVIDER_TPL_ID_LOCK1,
            provider_id: wf_provider.id().to_string(),
        })),
        Some(vec![Lock1::template_settings(None)]),
        vec![(&member, 1)],
        100,
        WfBasicPkg1::deposit_propose(),
        WfBasicPkg1::deposit_vote(),
        ProposalState::Accepted,
    )
    .await?;

    // Execute AddWorkflow by DAO member to add Lock1.
    run_activity(
        &worker,
        &member,
        &dao_account_id,
        proposal_id,
        1,
        ActivityInputWfBasicPkg1::activity_1(wf_provider.id(), PROVIDER_TPL_ID_LOCK1),
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

    // Propose Lock1.
    let proposal_id = proposal_to_finish(
        &worker,
        &member,
        &dao_account_id,
        DAO_TPL_ID_OF_FIRST_ADDED,
        Lock1::propose_settings(
            None,
            ActivityInputLock1::propose_settings_activity_1(token_account_id.as_str()),
        ),
        None,
        vec![(&member, 1)],
        100,
        Lock1::deposit_propose(),
        Lock1::deposit_vote(),
        ProposalState::Accepted,
    )
    .await?;

    // Execute workflow Lock1.
    run_activity(
        &worker,
        &member,
        &dao_account_id,
        proposal_id,
        1,
        ActivityInputLock1::activity_1(),
        true,
    )
    .await?;
    worker.wait(5).await?;
    debug_log(&worker, &dao_account_id).await?;
    statistics(&worker, &dao_account_id).await?;
    check_partitions(&worker, &dao_account_id, vec!["treasury partition name"]).await?;
    check_instance(
        &worker,
        &dao_account_id,
        proposal_id,
        1,
        1,
        InstanceState::Finished,
    )
    .await?;

    let proposal_id = proposal_to_finish(
        &worker,
        &member,
        &dao_account_id,
        DAO_TPL_ID_OF_FIRST_ADDED,
        Lock1::propose_settings(None, HashMap::new()),
        None,
        vec![(&member, 1)],
        100,
        Lock1::deposit_propose(),
        Lock1::deposit_vote(),
        ProposalState::Accepted,
    )
    .await?;
    check_partitions(&worker, &dao_account_id, vec!["treasury partition name"]).await?;

    // Add new FT to the created lock.
    run_activity(
        &worker,
        &member,
        &dao_account_id,
        proposal_id,
        2,
        ActivityInputLock1::activity_2_add_ft(3, 2, 9_999_999),
        true,
    )
    .await?;
    worker.wait(5).await?;
    debug_log(&worker, &dao_account_id).await?;
    statistics(&worker, &dao_account_id).await?;
    check_partitions(&worker, &dao_account_id, vec!["treasury partition name"]).await?;
    check_instance(
        &worker,
        &dao_account_id,
        proposal_id,
        2,
        1,
        InstanceState::Running,
    )
    .await?;
    Ok(())
}
