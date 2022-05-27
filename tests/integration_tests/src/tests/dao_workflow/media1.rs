use data::workflow::basic::media::Media1;
use data::workflow::basic::wf_add::{WfAdd1, WfAdd1ProposeOptions};
use near_sdk::ONE_NEAR;

use crate::constants::{DAO_TPL_ID_OF_FIRST_ADDED, DAO_TPL_ID_WF_ADD, PROVIDER_TPL_ID_MEDIA1};
#[allow(unused_imports)]
use crate::utils::{
    check_group_roles, check_instance, check_sale, check_user_roles, check_wf_storage_values,
    check_wf_templates, create_dao_via_factory, create_ft_via_factory, debug_log, ft_balance_of,
    init_dao_factory, init_ft_factory, init_skyward, init_staking, init_wnear,
    init_workflow_provider, load_workflow_templates, proposal_to_finish,
    proposal_to_finish_testnet, run_activity, statistics, storage_deposit, wf_log,
    ActivityInputWfAdd1, Wait,
};
use crate::{
    types::ProposalState,
    utils::{check_media, ActivityInputMedia1},
};
use library::workflow::instance::InstanceState;
use workspaces::{network::DevAccountDeployer, AccountId};

const DAO_FT_TOTAL_SUPPLY: u128 = 1_000_000_000;
const DEFAULT_DECIMALS: u128 = 10u128.pow(24);

#[tokio::test]
async fn workflow_media1_scenario() -> anyhow::Result<()> {
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
            template_id: PROVIDER_TPL_ID_MEDIA1,
            provider_id: wf_provider.id().to_string(),
        })),
        Some(vec![Media1::template_settings(None)]),
        vec![(&member, 1)],
        100,
        WfAdd1::deposit_propose(),
        WfAdd1::deposit_vote(),
        ProposalState::Accepted,
    )
    .await?;

    // Execute AddWorkflow by DAO member to add Media1.
    run_activity(
        &worker,
        &member,
        &dao_account_id,
        proposal_id,
        1,
        ActivityInputWfAdd1::activity_1(wf_provider.id(), PROVIDER_TPL_ID_MEDIA1),
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

    // Propose Media1.
    let proposal_id = proposal_to_finish(
        &worker,
        &member,
        &dao_account_id,
        DAO_TPL_ID_OF_FIRST_ADDED,
        Media1::propose_settings(
            None,
            ActivityInputMedia1::propose_settings_activity_1_cid(),
            ActivityInputMedia1::propose_settings_activity_2_cid(),
        ),
        None,
        vec![(&member, 1)],
        100,
        Media1::deposit_propose(),
        Media1::deposit_vote(),
        ProposalState::Accepted,
    )
    .await?;

    // Execute workflow Media1.
    run_activity(
        &worker,
        &member,
        &dao_account_id,
        proposal_id,
        1,
        ActivityInputMedia1::activity_1(),
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
    check_media(&worker, &dao_account_id, vec!["cid media name"]).await?;
    run_activity(
        &worker,
        &member,
        &dao_account_id,
        proposal_id,
        2,
        ActivityInputMedia1::activity_2(),
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
    debug_log(&worker, &dao_account_id).await?;
    statistics(&worker, &dao_account_id).await?;
    check_media(&worker, &dao_account_id, vec!["cid media name UPDATED"]).await?;

    Ok(())
}
