use std::collections::HashMap;

use data::{
    workflow::{
        basic::{
            bounty::Bounty1, group::Group1, group_package::GroupPackage1, lock::Lock1,
            media::Media1, reward::Reward1, trade::Trade1, wf_add::WfAdd1,
        },
        integration::skyward::{Skyward1, Skyward1TemplateOptions},
    },
    TemplateData,
};
use library::{
    types::source::SourceDataVariant,
    workflow::{help::TemplateHelp, template::Template},
    Version,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use workspaces::{network::DevAccountDeployer, AccountId, Contract, DevNetwork, Worker};

use crate::{
    test_data::WfOptionalActions,
    utils::{
        get_wf_provider_wasm, outcome_pretty, parse_view_result, view_outcome_pretty, FnCallId,
        MethodName,
    },
};

pub async fn init_workflow_provider<T>(worker: &Worker<T>) -> anyhow::Result<Contract>
where
    T: DevNetwork,
{
    let provider_blob_path = get_wf_provider_wasm();
    let provider = worker
        .dev_deploy(&std::fs::read(provider_blob_path)?)
        .await?;
    Ok(provider)
}

pub async fn load_workflow_templates<T>(
    worker: &Worker<T>,
    provider: &Contract,
    wnear_id: Option<&AccountId>,
    skyward_id: Option<&AccountId>,
) -> anyhow::Result<()>
where
    T: DevNetwork,
{
    let tpls = wf_templates(provider.id().to_string(), wnear_id, skyward_id);
    let templates_len = tpls.len();
    for (name, tpl, help) in tpls {
        let (wf, fncalls, meta, std_fncalls) = tpl;
        let args = json!({
            "workflow": wf,
            "fncalls": fncalls,
            "standard_fncalls": std_fncalls,
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
    provider_id: String,
    wnear_id: Option<&AccountId>,
    skyward_id: Option<&AccountId>,
) -> Vec<(String, TemplateData, Option<TemplateHelp>)> {
    let mut templates = vec![];
    templates.push(("wf_add".into(), WfAdd1::template(provider_id), None));
    if wnear_id.is_some() && skyward_id.is_some() {
        templates.push((
            "skyward1".into(),
            Skyward1::template(Some(Skyward1TemplateOptions {
                skyward_account_id: skyward_id.unwrap().to_string(),
                wnear_account_id: wnear_id.unwrap().to_string(),
            })),
            None,
        ));
    } else {
        templates.push(dummy_template_data());
    }
    templates.push(("trade1".into(), Trade1::template(), None));
    templates.push(("bounty1".into(), Bounty1::template(), None));
    templates.push(("reward1".into(), Reward1::template(), None));
    templates.push(("group_package1".into(), GroupPackage1::template(), None));
    templates.push((
        "test_optional_actions".into(),
        WfOptionalActions::template(),
        None,
    ));
    templates.push(("media1".into(), Media1::template(), None));
    templates.push(("lock1".into(), Lock1::template(), None));
    templates.push(("group1".into(), Group1::template(), None));
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

/// Makes padding so templates always have same id on provider.
fn dummy_template_data() -> (String, TemplateData, Option<TemplateHelp>) {
    (
        "dummy".into(),
        (
            Template {
                code: "dummy template".into(),
                version: "1.0".into(),
                auto_exec: false,
                need_storage: false,
                activities: vec![],
                expressions: vec![],
                transitions: vec![],
                constants: SourceDataVariant::Map(HashMap::default()),
                end: vec![],
            },
            vec![],
            vec![],
            vec![],
        ),
        None,
    )
}
