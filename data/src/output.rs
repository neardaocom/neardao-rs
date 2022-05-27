//! Workflow data which are outputed on stdout and loaded to workflow provider by hand.
//! TODO: Missing automation.

use library::{
    workflow::{help::TemplateHelp, template::Template, types::ObjectMetadata},
    FnCallId, MethodName,
};

use near_sdk::serde_json;

use crate::{
    object_metadata::standard_fn_calls::{standard_fn_call_metadatas, standard_fn_call_methods},
    workflow::{
        basic::{
            bounty::Bounty1, group::Group1, group_package::GroupPackage1, lock::Lock1,
            media::Media1, reward::Reward1, trade::Trade1, wf_add::WfAdd1,
        },
        integration::skyward::{Skyward1, Skyward1TemplateOptions},
    },
};

pub const WF_PROVIDER: &str = "workflow-provider.v1.neardao.testnet";
pub const SKYWARD: &str = "skyward.v1.neardao.testnet";
pub const WNEAR: &str = "wnear.v1.neardao.testnet";

fn pretty_print_template_data(
    name: &str,
    data: (
        (
            Template,
            Vec<FnCallId>,
            Vec<Vec<ObjectMetadata>>,
            Vec<MethodName>,
        ),
        Option<TemplateHelp>,
    ),
) {
    println!(
        "------------------------------ WF: {} ------------------------------\n{{\"workflow\":\n{},\n\"fncalls\":{},\n\"fncall_metadata\":{},\n\"standard_fncalls\":{},\n\"help\":{}}}",
        name,
        serde_json::to_string(&data.0.0).expect(name),
        serde_json::to_string(&data.0.1).expect(name),
        serde_json::to_string(&data.0.2).expect(name),
        serde_json::to_string(&data.0.3).expect(name),
        serde_json::to_string(&data.1).expect(name),
    );
}

fn pretty_print_standards(functions: Vec<MethodName>, metadata: Vec<Vec<ObjectMetadata>>) {
    println!(
        "------------------------------ STANDARD FNCALLS ------------------------------\n{{\"fncalls\":{},\"fncall_metadata\":{}}}",
        serde_json::to_string(&functions).unwrap(),
        serde_json::to_string(&metadata).unwrap(),
    );
}

#[test]
fn output_workflows_basic() {
    pretty_print_template_data("WFADD1", (WfAdd1::template(WF_PROVIDER.into()), None));
    pretty_print_template_data(
        "SKYWARD1",
        (
            Skyward1::template(Some(Skyward1TemplateOptions {
                skyward_account_id: SKYWARD.into(),
                wnear_account_id: WNEAR.into(),
            })),
            None,
        ),
    );
    pretty_print_template_data("BOUNTY1", (Bounty1::template(), None));
    pretty_print_template_data("REWARD1", (Reward1::template(), None));
    pretty_print_template_data("TRADE1", (Trade1::template(), None));
    pretty_print_template_data("MEDIA", (Media1::template(), None));
    pretty_print_template_data("LOCK1", (Lock1::template(), None));
    pretty_print_template_data("GROUP1", (Group1::template(), None));
    pretty_print_template_data("GROUP_PACKAGE1", (GroupPackage1::template(), None));
}

#[test]
fn output_standard_fn_calls() {
    let methods = standard_fn_call_methods();
    let fn_calls = standard_fn_call_metadatas();
    pretty_print_standards(methods, fn_calls);
}

#[test]
fn output_wf_add_template_settings() {
    println!(
        "------------------------------ WFADD1 - TEMPLATE SETTINGS ------------------------------\n{}",
        serde_json::to_string(&vec![WfAdd1::template_settings(Some(60))]).unwrap(),
    );
}
