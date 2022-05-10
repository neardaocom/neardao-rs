use std::collections::HashMap;

use data::{
    object_metadata::standard_fn_calls::{standard_fn_call_metadatas, standard_fn_call_methods},
    workflow::basic::wf_add::WfAdd1,
};
use library::{
    types::datatype::Datatype,
    workflow::{settings::TemplateSettings, template::Template, types::ObjectMetadata},
};
use workspaces::{network::DevAccountDeployer, Account, AccountId, Contract, DevNetwork, Worker};

use crate::{
    contract_utils::dao::check::internal_check_group,
    utils::{get_dao_wasm, outcome_pretty, FnCallId, MethodName},
};

use super::types::init::{
    DaoInit, DaoSettings, GroupInput, GroupMember, GroupOutput, GroupSettings,
};

pub(crate) async fn init_dao<T>(
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

    internal_check_group(worker, dao, expected_group).await?;
    // TODO: Other data checks - fn call, token.

    Ok(())
}

pub(crate) async fn deploy_dao<T>(worker: &Worker<T>) -> anyhow::Result<Contract>
where
    T: DevNetwork,
{
    let dao_blob_path = get_dao_wasm();
    let dao = worker.dev_deploy(&std::fs::read(dao_blob_path)?).await?;

    Ok(dao)
}

fn dao_init_args(
    token_id: AccountId,
    total_supply: u32,
    decimals: u8,
    staking_id: AccountId,
    provider_id: AccountId,
    admin_id: AccountId,
    council_members: Vec<&AccountId>,
) -> (DaoInit, GroupOutput) {
    let settings = dao_settings(provider_id.clone(), admin_id);
    let group = default_group(council_members);
    let standard_function_calls = standard_function_calls();
    let standard_function_call_metadata = standard_function_call_metadata();
    let function_calls = function_calls(provider_id);
    let function_call_metadata = function_call_metadata();
    let workflow_templates = workflow_templates();
    let workflow_template_settings = workflow_template_settings();

    let group_output = GroupOutput {
        settings: group.settings.clone(),
        id: 1,
        members: group.members.clone(),
    };
    (
        DaoInit {
            token_id,
            staking_id,
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
            tick_interval: 3600,
        },
        group_output,
    )
}

fn dao_settings(provider_id: AccountId, admin_id: AccountId) -> DaoSettings {
    DaoSettings {
        name: "Test dao".into(),
        purpose: "testing".into(),
        tags: vec![],
        dao_admin_account_id: admin_id,
        dao_admin_rights: vec!["all".into()],
        workflow_provider: provider_id,
    }
}
fn default_group(members: Vec<&AccountId>) -> GroupInput {
    let leader = AccountId::try_from(members[0].to_string()).unwrap();
    let mut member_roles = HashMap::new();
    member_roles.insert(
        "council".into(),
        members
            .iter()
            .map(|m| m.to_string())
            .collect::<Vec<String>>(),
    );
    let members = members
        .into_iter()
        .map(|m| GroupMember {
            account_id: m.to_owned(),
            tags: vec![],
        })
        .collect();
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

fn function_calls(provider_id: AccountId) -> Vec<FnCallId> {
    let calls = vec![(provider_id, "wf_template".to_string())];
    calls
}

fn standard_function_call_metadata() -> Vec<Vec<ObjectMetadata>> {
    standard_fn_call_metadatas()
}

fn function_call_metadata() -> Vec<Vec<ObjectMetadata>> {
    let meta = vec![vec![ObjectMetadata {
        arg_names: vec!["id".into()],
        arg_types: vec![Datatype::U64(false)],
    }]];
    meta
}

fn workflow_templates() -> Vec<Template> {
    let tpls = vec![WfAdd1::template()];
    tpls
}

fn workflow_template_settings() -> Vec<Vec<TemplateSettings>> {
    let settings = vec![vec![WfAdd1::template_settings(Some(10))]];
    settings
}
