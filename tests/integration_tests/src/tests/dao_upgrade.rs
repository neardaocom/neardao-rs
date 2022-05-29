#[allow(unused_imports)]
use crate::utils::{
    check_group_roles, check_instance, check_sale, check_user_roles, check_wf_storage_values,
    check_wf_templates, create_dao_via_factory, create_ft_via_factory, debug_log, ft_balance_of,
    init_dao_factory, init_ft_factory, init_skyward, init_staking, init_wnear,
    init_workflow_provider, load_workflow_templates, proposal_to_finish,
    proposal_to_finish_testnet, run_activity, statistics, storage_deposit, wf_log,
    ActivityInputSkyward1, ActivityInputWfAdd1, Wait,
};
use crate::utils::{
    deploy_upgrade_dao_factory,
    upgrade::{
        dao_add_dummy_data, dao_download_new_version_and_start_migration, dao_migrate_data,
        dao_upgrade, dao_view_dummy_data, dao_view_dummy_data_after_migration,
    },
};

use workspaces::{network::DevAccountDeployer, AccountId};

const DAO_FT_TOTAL_SUPPLY: u128 = 1_000_000_000;
const DEFAULT_DECIMALS: u128 = 10u128.pow(24);

/// Test upgrade process for DAO.
#[ignore = "Test only when something with upgrade process change."]
#[tokio::test]
async fn dao_upgrade_process() -> anyhow::Result<()> {
    let dao_name = "test_dao";
    let worker = workspaces::sandbox().await?;
    let member = worker.dev_create_account().await?;

    let factory = deploy_upgrade_dao_factory(&worker, None, "v1").await?;
    let dao_account_id = AccountId::try_from(format!("{}.{}", dao_name, factory.id()))
        .expect("invalid dao account id");
    let token_account_id =
        AccountId::try_from("dao_token.near".to_string()).expect("invalid ft account id");
    let staking_account_id =
        AccountId::try_from("staking.near".to_string()).expect("invalid staking account id");
    let wf_provider_account_id = AccountId::try_from("wf-provider.near".to_string())
        .expect("invalid wf-provider account id");

    create_dao_via_factory(
        &worker,
        &factory,
        &dao_name,
        &token_account_id,
        DAO_FT_TOTAL_SUPPLY as u32,
        24,
        &staking_account_id,
        &wf_provider_account_id,
        factory.id(),
        vec![member.id()],
        0,
    )
    .await?;

    dao_add_dummy_data(
        &worker,
        &member,
        &dao_account_id,
        vec!["first,second,third"],
    )
    .await?;
    dao_view_dummy_data(&worker, &dao_account_id).await?;
    statistics(&worker, &dao_account_id).await?;

    deploy_upgrade_dao_factory(&worker, Some(factory.as_account()), "v2_migration").await?;
    deploy_upgrade_dao_factory(&worker, Some(factory.as_account()), "v2").await?;

    dao_download_new_version_and_start_migration(&worker, &member, &dao_account_id).await?;
    dao_migrate_data(&worker, &member, &dao_account_id).await?;
    dao_upgrade(&worker, &member, &dao_account_id).await?;

    // Test all works.
    dao_view_dummy_data_after_migration(&worker, &dao_account_id).await?;
    dao_add_dummy_data(&worker, &member, &dao_account_id, vec!["aaa,bbb,ccc"]).await?;
    dao_view_dummy_data_after_migration(&worker, &dao_account_id).await?;
    statistics(&worker, &dao_account_id).await?;
    Ok(())
}
