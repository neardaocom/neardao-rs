use near_sdk::ONE_NEAR;

use crate::constants::{DAO_TPL_ID_OF_FIRST_ADDED, DAO_TPL_ID_WF_ADD, PROVIDER_TPL_ID_TRADE1};
use crate::{
    types::ProposalState,
    utils::{
        check_instance, check_wf_storage_values, check_wf_templates, create_dao_via_factory,
        create_ft_via_factory, debug_log, ft_balance_of, ft_transfer_call, init_dao_factory,
        init_ft_factory, init_staking, init_workflow_provider, load_workflow_templates,
        proposal_to_finish, run_activity, serialized_dao_ft_receiver_workflow_msg, storage_deposit,
        ActivityInputTrade1, ActivityInputWfAdd1, Wait,
    },
};

use data::workflow::basic::{
    trade::{Trade1, Trade1ProposeOptions},
    wf_add::{WfAdd1, WfAdd1ProposeOptions},
};
use library::workflow::instance::InstanceState;
use workspaces::{network::DevAccountDeployer, AccountId};

const DAO_FT_TOTAL_SUPPLY: u128 = 1_000_000_000;
const DEFAULT_DECIMALS: u128 = 10u128.pow(24);

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
        serialized_dao_ft_receiver_workflow_msg(2, "trade"),
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
        serialized_dao_ft_receiver_workflow_msg(proposal_id, "trade"),
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
        0,
        InstanceState::Running,
    )
    .await?;
    Ok(())
}
