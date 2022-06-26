use data::workflow::integration::skyward::ONE_MONTH;
use near_sdk::ONE_NEAR;

use crate::constants::{DAO_TPL_ID_OF_FIRST_ADDED, DAO_TPL_ID_WF_ADD, PROVIDER_TPL_ID_SKYWARD1};
use crate::types::{ProposalState, Roles, UserRoles};
#[allow(unused_imports)]
use crate::utils::{
    check_group_roles, check_instance, check_sale, check_user_roles, check_wf_storage_values,
    check_wf_templates, create_dao_via_factory, create_ft_via_factory, debug_log, ft_balance_of,
    init_dao_factory, init_ft_factory, init_skyward, init_staking, init_wnear,
    init_workflow_provider, load_workflow_templates, proposal_to_finish,
    proposal_to_finish_testnet, run_activity, statistics, storage_deposit, wf_log,
    ActivityInputSkyward1, ActivityInputWfBasicPkg1, Wait,
};
use data::workflow::{
    basic::basic_package::{WfBasicPkg1, WfBasicPkg1ProposeOptions},
    integration::skyward::{Skyward1, Skyward1ProposeOptions, AUCTION_START, ONE_WEEK},
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
        WfBasicPkg1::propose_settings(Some(WfBasicPkg1ProposeOptions {
            template_id: PROVIDER_TPL_ID_SKYWARD1,
            provider_id: wf_provider.id().to_string(),
        })),
        Some(vec![Skyward1::template_settings()]),
        vec![(&member, 1)],
        100,
        WfBasicPkg1::deposit_propose(),
        WfBasicPkg1::deposit_vote(),
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
        ActivityInputWfBasicPkg1::activity_1(wf_provider.id(), PROVIDER_TPL_ID_SKYWARD1),
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
                auction_start: AUCTION_START + 5 * ONE_MONTH,
                auction_duration: ONE_WEEK,
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
        1,
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
    wf_log(&worker, &dao_account_id, proposal_id).await?;

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
        1,
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
                auction_start: AUCTION_START + 5 * ONE_MONTH,
                auction_duration: ONE_WEEK,
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
        1,
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
        1,
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
