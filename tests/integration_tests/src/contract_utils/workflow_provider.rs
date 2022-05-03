use library::{
    data::{
        skyward::{workflow_skyward_template_data_1, SkywardTemplateDataOptions},
        TemplateData,
    },
    workflow::help::TemplateHelp,
};
use serde_json::json;
use workspaces::{network::DevAccountDeployer, Account, AccountId, Contract, DevNetwork, Worker};

use crate::utils::{get_wf_provider_wasm, outcome_pretty};

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
    provider_id: &Contract,
    wnear_id: &AccountId,
    skyward_id: &AccountId,
) -> anyhow::Result<()>
where
    T: DevNetwork,
{
    let tpls = wf_templates(wnear_id, skyward_id);
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
        let outcome = provider_id
            .as_account()
            .call(&worker, provider_id.id(), "workflow_add")
            .args(args)
            .max_gas()
            .transact()
            .await?;
        let title = format!("wf provider add workflow: {name}");
        outcome_pretty(&title, &outcome);
        assert!(outcome.is_success(), "wf provider add workflows failed.");
    }
    Ok(())
}

fn wf_templates(
    wnear_id: &AccountId,
    skyward_id: &AccountId,
) -> Vec<(String, TemplateData, Option<TemplateHelp>)> {
    let templates = vec![(
        "skyward".into(),
        workflow_skyward_template_data_1(Some(SkywardTemplateDataOptions {
            skyward_account_id: skyward_id.to_string(),
            wnear_account_id: wnear_id.to_string(),
        })),
        None,
    )];
    templates
}
