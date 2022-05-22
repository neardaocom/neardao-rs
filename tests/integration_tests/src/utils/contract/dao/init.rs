use std::str::FromStr;

use data::{
    object_metadata::standard_fn_calls::{standard_fn_call_metadatas, standard_fn_call_methods},
    workflow::basic::wf_add::WfAdd1,
};
use library::{
    locking::{LockInput, UnlockMethod, UnlockPeriodInput, UnlockingInput},
    workflow::{settings::TemplateSettings, template::Template, types::ObjectMetadata},
};
use workspaces::{network::DevAccountDeployer, Account, AccountId, Contract, DevNetwork, Worker};

use crate::utils::{as_account_id, get_dao_wasm, outcome_pretty, FnCallId, MethodName};

use crate::types::{
    Asset, {DaoInit, DaoSettings, PartitionAssetInput, TreasuryPartitionInput},
    {Group, GroupInput, GroupMember, GroupMembers, GroupSettings, MemberRoles},
};

/// Deprecated.
/// Used init via factory instead.
pub async fn init_dao<T>(
    worker: &Worker<T>,
    factory: &Account,
    dao: &Contract,
    token_id: &AccountId,
    total_supply: u32,
    decimals: u8,
    staking_id: &AccountId,
    provider_id: &AccountId,
    admin_id: &AccountId,
    council_members: Vec<&AccountId>,
    init_distribution: u32,
) -> anyhow::Result<()>
where
    T: DevNetwork,
{
    let (init_args, expected_group) = dao_init_args(
        token_id.clone(),
        total_supply,
        decimals,
        staking_id.clone(),
        provider_id.clone(),
        admin_id.clone(),
        council_members,
        init_distribution,
    );
    let args = serde_json::to_string(&init_args)
        .expect("Failed to serialize DaoInit object")
        .into_bytes();
    let outcome = factory
        .call(&worker, dao.id(), "new")
        .args(args)
        .max_gas()
        .transact()
        .await?;
    outcome_pretty::<()>("dao init", &outcome);
    assert!(outcome.is_success(), "dao init failed");

    //internal_check_group(worker, dao, expected_group).await?;
    // TODO: Other data checks - fn call, token.

    Ok(())
}

pub async fn deploy_dao<T>(worker: &Worker<T>) -> anyhow::Result<Contract>
where
    T: DevNetwork,
{
    let dao_blob_path = get_dao_wasm();
    let dao = worker.dev_deploy(&std::fs::read(dao_blob_path)?).await?;

    Ok(dao)
}

pub fn dao_init_args(
    token_id: AccountId,
    total_supply: u32,
    decimals: u8,
    staking_id: AccountId,
    provider_id: AccountId,
    admin_id: AccountId,
    council_members: Vec<&AccountId>,
    init_distribution: u32,
) -> (DaoInit, (u16, Group)) {
    let settings = dao_settings(provider_id.clone(), admin_id, token_id.clone(), staking_id);
    let group = default_group(council_members);
    let standard_function_calls = standard_function_calls();
    let standard_function_call_metadata = standard_function_call_metadata();
    let function_calls = function_calls(provider_id.to_string());
    let function_call_metadata = function_call_metadata(provider_id.to_string());
    let workflow_templates = workflow_templates(provider_id.to_string());
    let workflow_template_settings = workflow_template_settings();
    let treasury_partitions =
        treasury_partitions(token_id.as_str(), total_supply - init_distribution);

    let members = group
        .members
        .clone()
        .into_iter()
        .map(|m| (m.account_id, m.tags))
        .collect();
    let group_output = (
        1,
        Group {
            settings: group.settings.clone(),
            members: GroupMembers(members),
            rewards: vec![],
        },
    );
    (
        DaoInit {
            total_supply,
            decimals,
            settings,
            groups: vec![group],
            tags: vec![],
            standard_function_calls,
            standard_function_call_metadata,
            function_calls,
            function_call_metadata,
            workflow_templates,
            workflow_template_settings,
            treasury_partitions,
        },
        group_output,
    )
}

fn dao_settings(
    provider_id: AccountId,
    admin_id: AccountId,
    token_id: AccountId,
    staking_id: AccountId,
) -> DaoSettings {
    DaoSettings {
        name: "Test dao".into(),
        purpose: "testing".into(),
        tags: vec![],
        dao_admin_account_id: admin_id,
        dao_admin_rights: vec!["all".into()],
        workflow_provider: provider_id,
        resource_provider: AccountId::from_str("resource-provider.neardao.testnet").unwrap(),
        scheduler: AccountId::from_str("scheduler.neardao.testnet").unwrap(),
        token_id,
        staking_id,
    }
}
fn default_group(members: Vec<&AccountId>) -> GroupInput {
    let leader = AccountId::try_from(members[0].to_string()).unwrap();
    let members_accounts = members
        .iter()
        .map(|m| m.to_string())
        .collect::<Vec<String>>();

    let members = members
        .into_iter()
        .map(|m| GroupMember {
            account_id: m.clone(),
            tags: vec![],
        })
        .collect::<Vec<GroupMember>>();

    let member_roles = vec![MemberRoles {
        name: "council".into(),
        members: members_accounts,
    }];

    GroupInput {
        settings: GroupSettings {
            name: "council".into(),
            leader: Some(leader),
            parent_group: 0,
        },
        members,
        member_roles,
    }
}

fn standard_function_calls() -> Vec<MethodName> {
    standard_fn_call_methods()
}

fn function_calls(provider_id: String) -> Vec<FnCallId> {
    WfAdd1::template(provider_id.to_string()).1
}

fn standard_function_call_metadata() -> Vec<Vec<ObjectMetadata>> {
    standard_fn_call_metadatas()
}

fn function_call_metadata(provider_id: String) -> Vec<Vec<ObjectMetadata>> {
    WfAdd1::template(provider_id).2
}

fn workflow_templates(provider_id: String) -> Vec<Template> {
    let tpls = vec![WfAdd1::template(provider_id).0];
    tpls
}

fn workflow_template_settings() -> Vec<Vec<TemplateSettings>> {
    let settings = vec![vec![WfAdd1::template_settings(Some(10))]];
    settings
}
fn treasury_partitions(
    vote_token_id: &str,
    vote_locked_amount: u32,
) -> Vec<TreasuryPartitionInput> {
    vec![
        TreasuryPartitionInput {
            name: "near_partition".into(),
            assets: vec![PartitionAssetInput {
                asset_id: Asset::new_near(),
                unlocking: UnlockingInput {
                    amount_init_unlock: 100,
                    lock: None,
                },
            }],
        },
        TreasuryPartitionInput {
            name: "vote_token_partition".into(),
            assets: vec![PartitionAssetInput {
                asset_id: Asset::new_ft(as_account_id(vote_token_id), 24),
                unlocking: UnlockingInput {
                    amount_init_unlock: 0,
                    lock: Some(LockInput {
                        amount_total_lock: vote_locked_amount as u32,
                        start_from: 0,
                        duration: 1000,
                        periods: vec![UnlockPeriodInput {
                            r#type: UnlockMethod::Linear,
                            duration: 1000,
                            amount: vote_locked_amount as u32,
                        }],
                    }),
                },
            }],
        },
    ]
}
