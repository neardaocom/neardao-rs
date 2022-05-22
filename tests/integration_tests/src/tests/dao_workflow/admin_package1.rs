use near_sdk::ONE_NEAR;

use crate::constants::{
    DAO_TPL_ID_OF_FIRST_ADDED, DAO_TPL_ID_WF_ADD, PROVIDER_TPL_ID_ADMIN_PACKAGE1,
};
use crate::types::{ProposalState, Roles, UserRoles};
use crate::utils::{
    check_group, check_group_exists, check_group_roles, check_instance, check_user_roles,
    check_wf_templates, create_dao_via_factory, create_ft_via_factory, init_dao_factory,
    init_ft_factory, init_staking, init_workflow_provider, load_workflow_templates,
    proposal_to_finish, run_activity, storage_deposit, view_groups, view_partitions,
    ActivityInputAdminPkg1, ActivityInputWfAdd1, Wait, ADMINPACKAGE1_ADD_GROUP,
    ADMINPACKAGE1_ADD_GROUP_MEMBERS, ADMINPACKAGE1_REMOVE_GROUP,
    ADMINPACKAGE1_REMOVE_GROUP_MEMBERS, ADMINPACKAGE1_REMOVE_GROUP_MEMBER_ROLES,
    ADMINPACKAGE1_REMOVE_GROUP_ROLES,
};

use data::workflow::basic::{
    admin_package::AdminPackage1,
    wf_add::{WfAdd1, WfAdd1ProposeOptions},
};
use library::workflow::instance::InstanceState;
use workspaces::{network::DevAccountDeployer, AccountId};

const DAO_FT_TOTAL_SUPPLY: u128 = 1_000_000_000;
const DEFAULT_DECIMALS: u128 = 10u128.pow(24);

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
