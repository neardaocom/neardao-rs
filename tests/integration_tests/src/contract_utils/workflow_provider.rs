use library::{
    data::{
        workflows::integration::skyward::{Skyward1, Skyward1TemplateOptions},
        TemplateData,
    },
    workflow::help::TemplateHelp,
    Version,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use workspaces::{network::DevAccountDeployer, AccountId, Contract, DevNetwork, Worker};

use crate::utils::{
    get_wf_provider_wasm, outcome_pretty, parse_view_result, view_outcome_pretty, FnCallId,
    MethodName,
};

pub(crate) async fn init_workflow_provider<T>(worker: &Worker<T>) -> anyhow::Result<Contract>
where
    T: DevNetwork,
{
    let provider_blob_path = get_wf_provider_wasm();
    let provider = worker
        .dev_deploy(&std::fs::read(provider_blob_path)?)
        .await?;
    Ok(provider)
}

pub(crate) async fn load_workflow_templates<T>(
    worker: &Worker<T>,
    provider: &Contract,
    wnear_id: &AccountId,
    skyward_id: &AccountId,
) -> anyhow::Result<()>
where
    T: DevNetwork,
{
    let tpls = wf_templates(wnear_id, skyward_id);
    let templates_len = tpls.len();
    for (name, tpl, help) in tpls {
        let (wf, fncalls, meta) = tpl;
        let args = json!({
            "workflow": wf,
            "fncalls": fncalls,
            "fncall_metadata": meta,
            "help": help,
        })
        .to_string()
        .into_bytes();
        let outcome = provider
            .as_account()
            .call(&worker, provider.id(), "workflow_add")
            .args(args)
            .max_gas()
            .transact()
            .await?;
        let title = format!("wf provider add workflow: {name}");
        outcome_pretty::<u16>(&title, &outcome);
        assert!(outcome.is_success(), "wf provider add workflows failed.");
    }

    let templates = template_metadatas(worker, provider).await?;
    assert_eq!(
        templates_len,
        templates.len(),
        "provider: count of loaded templates do not match count of actually stored"
    );

    Ok(())
}

fn wf_templates(
    wnear_id: &AccountId,
    skyward_id: &AccountId,
) -> Vec<(String, TemplateData, Option<TemplateHelp>)> {
    let templates = vec![(
        "skyward".into(),
        Skyward1::template(Some(Skyward1TemplateOptions {
            skyward_account_id: skyward_id.to_string(),
            wnear_account_id: wnear_id.to_string(),
        })),
        None,
    )];
    templates
}

async fn template_metadatas<T>(
    worker: &Worker<T>,
    provider: &Contract,
) -> anyhow::Result<Vec<Metadata>>
where
    T: DevNetwork,
{
    let args = json!({}).to_string().into_bytes();
    let outcome = provider.view(&worker, "wf_templates", args).await?;
    view_outcome_pretty::<Vec<Metadata>>("provider check_templates", &outcome);
    Ok(parse_view_result::<Vec<Metadata>>(&outcome).expect("failed to parse provider's templates"))
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Metadata {
    pub id: u16,
    pub code: String,
    pub version: Version,
    pub fncalls: Vec<FnCallId>,
    pub standard_fncalls: Vec<MethodName>,
    pub help: bool,
}
