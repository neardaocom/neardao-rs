//! Simple workflow data generator which helps to automate manual loader script updating.

use library::{
    workflow::{settings::TemplateSettings, template::Template, types::ObjectMetadata},
    FnCallId, MethodName,
};
use std::{
    fs::OpenOptions,
    io::{Error, Read},
    os::unix::prelude::{FileExt, MetadataExt},
    path::Path,
};

use near_sdk::serde_json;

use data::{
    object_metadata::standard_fn_calls::{standard_fn_call_metadatas, standard_fn_call_methods},
    workflow::{
        basic::{
            basic_package::WfBasicPkg1,
            bounty::Bounty1,
            group::Group1,
            group_package::GroupPackage1,
            lock::Lock1,
            media::Media1,
            reward::{Reward1, Reward2},
            trade::Trade1,
        },
        integration::skyward::{Skyward1, Skyward1TemplateOptions},
    },
    TemplateData,
};

const BASE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../scripts/");
const FILE_NAME: &str = "wf_provider_data_loader.sh";
const DATA_BEGIN_SENTINEL: &str = "# ------------ DATA ------------ #";
const DATA_END_SENTINEL: &str = "# ------------ LOAD ------------ #";

pub const WF_PROVIDER: &str = "workflow-provider.v1.neardao.testnet";
pub const SKYWARD: &str = "skyward.v1.neardao.testnet";
pub const WNEAR: &str = "wnear.v1.neardao.testnet";

fn main() -> Result<(), Box<Error>> {
    let script_path = format!("{}{}", BASE_PATH, FILE_NAME);
    update_loader_data(script_path)
}

fn update_loader_data<P: AsRef<Path>>(script_path: P) -> Result<(), Box<Error>> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(script_path)?;
    let mut buf = String::with_capacity(
        file.metadata()
            .expect("Failed to read file's metadata.")
            .size() as usize,
    );
    file.read_to_string(&mut buf)?;
    let begin_start = buf
        .find(DATA_BEGIN_SENTINEL)
        .expect("Data begin sentinel not found.")
        + DATA_BEGIN_SENTINEL.len()
        + 2;
    let end_start = buf
        .find(DATA_END_SENTINEL)
        .expect("Data end sentinel not found.");
    let data_buf = generate_workflow_data();
    file.set_len(begin_start as u64)?;
    file.write_all_at(data_buf.as_bytes(), begin_start as u64)?;
    file.write_all_at(
        &buf.as_bytes()[end_start..],
        begin_start as u64 + data_buf.len() as u64,
    )?;
    Ok(())
}

/// Generate data string for the loader script.
fn generate_workflow_data() -> String {
    let mut buf = String::with_capacity(2usize.pow(16));
    for (idx, (name, tpl_data)) in wf_templates().into_iter().enumerate() {
        template_workflow_data(&mut buf, idx + 1, name, tpl_data);
    }
    buf.push_str("# Another data necessary for provider with NEAR-CLI scripts.\n\n");
    let methods = standard_fn_call_methods();
    let fn_calls = standard_fn_call_metadatas();
    template_standard_fncalls(&mut buf, methods, fn_calls);
    buf.push_str("# near call $WID standard_fncalls_add $STANDARD_FN_CALLS --accountId=$WID\n");
    template_basic_package_settings(&mut buf, WfBasicPkg1::template_settings(Some(60)));
    buf.push_str("# near call $WID wf_basic_package_add_settings '{\"settings\": '$WF_BASIC_PKG_TEMPLATE_SETTINGS'}' --accountId=$WID\n\n");
    buf
}

fn template_workflow_data(
    buf: &mut String,
    id: usize,
    name: &str,
    data: (
        Template,
        Vec<FnCallId>,
        Vec<Vec<ObjectMetadata>>,
        Vec<MethodName>,
    ),
) {
    let tpl = serde_json::to_string(&data.0).expect(name);
    let fns = serde_json::to_string(&data.1).expect(name);
    let fn_meta = serde_json::to_string(&data.2).expect(name);
    let std_fns = serde_json::to_string(&data.3).expect(name);
    buf.push_str(&format!(
        "# WFT - {}\nWF{}='{}'\nWF{}FNS='{}'\nWF{}FNMETA='{}'\nWF{}STDFNS='{}'\n\n",
        name, id, tpl, id, fns, id, fn_meta, id, std_fns
    ));
}

fn template_standard_fncalls(
    buf: &mut String,
    functions: Vec<MethodName>,
    metadata: Vec<Vec<ObjectMetadata>>,
) {
    buf.push_str("# STANDARD_FN_CALLS=");
    buf.push_str(&format!(
        "'{{\"fncalls\":{},\"fncall_metadata\":{}}}'\n",
        serde_json::to_string(&functions).unwrap(),
        serde_json::to_string(&metadata).unwrap(),
    ));
}

fn template_basic_package_settings(
    buf: &mut String,
    basic_pkg_template_settings: TemplateSettings,
) {
    let settings = serde_json::to_string(&basic_pkg_template_settings).unwrap();
    buf.push_str("# WF_BASIC_PKG_TEMPLATE_SETTINGS=");
    buf.push_str(&format!("'{}'\n", settings.as_str()));
}

fn wf_templates() -> Vec<(&'static str, TemplateData)> {
    let mut vec = Vec::with_capacity(64);
    vec.push(("BASIC_PACKAGE1", WfBasicPkg1::template(WF_PROVIDER.into())));
    vec.push((
        "SKYWARD1",
        Skyward1::template(Some(Skyward1TemplateOptions {
            skyward_account_id: SKYWARD.into(),
            wnear_account_id: WNEAR.into(),
        })),
    ));
    vec.push(("BOUNTY1", Bounty1::template()));
    vec.push(("REWARD1", Reward1::template()));
    vec.push(("TRADE1", Trade1::template()));
    vec.push(("MEDIA", Media1::template()));
    vec.push(("LOCK1", Lock1::template()));
    vec.push(("GROUP1", Group1::template()));
    vec.push(("GROUP_PACKAGE1", GroupPackage1::template()));
    vec.push(("REWARD2", Reward2::template()));
    vec
}
