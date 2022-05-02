use library::{
    data::standard_fn_calls::{
        nep_141_ft_transfer, nep_141_ft_transfer_call, nep_145_storage_deposit,
        nep_145_storage_unregister, nep_145_storage_withdraw, nep_171_nft_transfer,
        nep_171_nft_transfer_call,
    },
    types::datatype::Datatype,
    workflow::{settings::TemplateSettings, template::Template, types::ObjectMetadata},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use workspaces::{network::DevAccountDeployer, Account, AccountId, Contract, DevNetwork, Worker};

use crate::utils::{get_dao_wasm, outcome_pretty, DurationSec, FnCallId, MethodName};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct DaoInit {
    staking_id: AccountId,
    total_supply: u32,
    settings: DaoSettings,
    groups: Vec<GroupInput>,
    tags: Vec<u16>,
    standard_function_calls: Vec<MethodName>,
    standard_function_call_metadata: Vec<Vec<ObjectMetadata>>,
    function_calls: Vec<FnCallId>,
    function_call_metadata: Vec<Vec<ObjectMetadata>>,
    workflow_templates: Vec<Template>,
    workflow_template_settings: Vec<Vec<TemplateSettings>>,
    tick_interval: DurationSec,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct DaoSettings {
    pub name: String,
    pub purpose: String,
    pub tags: Vec<u16>,
    pub dao_admin_account_id: AccountId,
    pub dao_admin_rights: Vec<String>, //TODO should be rights
    pub workflow_provider: AccountId,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct GroupInput {
    pub settings: GroupSettings,
    pub members: Vec<GroupMember>,
    pub token_lock: Option<GroupTokenLockInput>,
}
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct GroupSettings {
    pub name: String,
    pub leader: Option<AccountId>,
    pub parent_group: u16,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct GroupMember {
    pub account_id: AccountId,
    pub tags: Vec<u16>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct GroupTokenLockInput {
    pub amount: u32,
    pub start_from: u64,
    pub duration: u64,
    pub init_distribution: u32,
    pub unlock_interval: u32,
    pub periods: Vec<UnlockPeriodInput>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum UnlockMethod {
    /// All FT immediately unlocked.
    None = 0,
    /// Linear unlocker over specified time period.
    Linear,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct UnlockPeriodInput {
    pub kind: UnlockMethod,
    pub duration: u64,
    pub amount: u32,
}

pub(crate) async fn init_dao<T>(
    worker: &Worker<T>,
    factory: &Account,
    dao_contract: &Contract,
    token_id: &AccountId,
    total_supply: u32,
    staking_id: &AccountId,
    provider_id: &AccountId,
    admin_id: &AccountId,
    council_members: Vec<&AccountId>,
) -> anyhow::Result<()>
where
    T: DevNetwork,
{
    let args = json!({}).to_string().into_bytes();
    let outcome = factory
        .call(&worker, dao_contract.id(), "new")
        .args(args)
        .max_gas()
        .transact()
        .await?;
    outcome_pretty("dao init", &outcome);
    assert!(outcome.is_success(), "dao init failed");
    Ok(())
}

pub(crate) async fn deploy_dao<T>(worker: &Worker<T>, factory: &Account) -> anyhow::Result<Contract>
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
    staking_id: AccountId,
    provider_id: AccountId,
    admin_id: AccountId,
    council_members: Vec<&AccountId>,
) -> DaoInit {
    let settings = dao_settings(provider_id.clone(), admin_id);
    let group = default_group(council_members);
    let standard_function_calls = standard_function_calls();
    let standard_function_call_metadata = standard_function_call_metadata();
    let function_calls = function_calls(provider_id);
    let function_call_metadata = function_call_metadata();
    let workflow_templates = workflow_templates();
    let workflow_template_settings = workflow_template_settings();

    DaoInit {
        staking_id,
        total_supply,
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
    }
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
        token_lock: None,
    }
}

fn standard_function_calls() -> Vec<MethodName> {
    let calls = vec![
        "nep_141_ft_transfer".into(),
        "nep_141_ft_transfer_call".into(),
        "nep_145_storage_deposit".into(),
        "nep_145_storage_unregister".into(),
        "nep_145_storage_withdraw".into(),
        "nep_171_nft_transfer".into(),
        "nep_171_nft_transfer_call".into(),
    ];

    calls
}

fn function_calls(provider_id: AccountId) -> Vec<FnCallId> {
    let calls = vec![(provider_id, "wf_template".to_string())];

    calls
}

fn standard_function_call_metadata() -> Vec<Vec<ObjectMetadata>> {
    let meta = vec![
        nep_141_ft_transfer(),
        nep_141_ft_transfer_call(),
        nep_145_storage_deposit(),
        nep_145_storage_unregister(),
        nep_145_storage_withdraw(),
        nep_171_nft_transfer(),
        nep_171_nft_transfer_call(),
    ];
    meta
}

fn function_call_metadata() -> Vec<Vec<ObjectMetadata>> {
    let meta = vec![vec![ObjectMetadata {
        arg_names: vec!["id".into()],
        arg_types: vec![Datatype::U64(false)],
    }]];

    meta
}

fn workflow_templates() -> Vec<Template> {
    let tpls = vec![];

    tpls
}

fn workflow_template_settings() -> Vec<Vec<TemplateSettings>> {
    let settings = vec![vec![]];

    settings
}
