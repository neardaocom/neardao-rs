//! Workflow data which are outputed on stdout and loaded to workflow provider by hand.
//! TODO: Missing automation.

use library::workflow::types::ObjectMetadata;

use near_sdk::serde_json;

use super::object_metadata::standard_fn_calls::{
    nep_141_ft_transfer, nep_141_ft_transfer_call, nep_171_nft_transfer, nep_171_nft_transfer_call,
};

fn pretty_print(name: &str, meta: Vec<ObjectMetadata>) {
    println!(
        "------------------------------ {} ------------------------------\n{}",
        name,
        serde_json::to_string(&meta).expect(name)
    );
}

/*
#[test]
fn output_workflows_basic() {
    println!(
        "------------------------------ WORKFLOW ADD ------------------------------\n{}",
        serde_json::to_string(&workflow_wf_add()).unwrap()
    );

    println!(
            "------------------------------ WORKFLOW PAYOUT NEAR IN LOOP ------------------------------\n{}",
            serde_json::to_string(&workflow_treasury_send_near_loop()).unwrap()
        );

    println!(
        "------------------------------ WORKFLOW PAYOUT NEAR ------------------------------\n{}",
        serde_json::to_string(&workflow_treasury_send_near()).unwrap()
    );

    println!(
        "------------------------------ WORKFLOW PAYOUT FT ------------------------------\n{}",
        serde_json::to_string(&workflow_treasury_send_ft()).unwrap()
    );

    println!(
        "------------------------------ WORKFLOW ADD GROUP ------------------------------\n{}",
        serde_json::to_string(&workflow_group_add()).unwrap()
    );

    println!(
            "------------------------------ WORKFLOW ADD GROUP MEMBERS ------------------------------\n{}",
            serde_json::to_string(&workflow_group_members_add()).unwrap()
        );

    println!(
            "------------------------------ WORKFLOW REMOVE GROUP MEMBER ------------------------------\n{}",
            serde_json::to_string(&workflow_group_member_remove()).unwrap()
        );

    println!(
        "------------------------------ WORKFLOW REMOVE GROUP ------------------------------\n{}",
        serde_json::to_string(&workflow_group_remove()).unwrap()
    );

    println!(
        "------------------------------ WORKFLOW TAG ADD ------------------------------\n{}",
        serde_json::to_string(&workflow_tag_add()).unwrap()
    );

    println!(
        "------------------------------ WORKFLOW TAG EDIT ------------------------------\n{}",
        serde_json::to_string(&workflow_tag_edit()).unwrap()
    );

    println!(
        "------------------------------ WORKFLOW FT DISTRIBUTE ------------------------------\n{}",
        serde_json::to_string(&workflow_ft_distribute()).unwrap()
    );

    println!(
        "------------------------------ WORKFLOW MEDIA ADD ------------------------------\n{}",
        serde_json::to_string(&workflow_media_add()).unwrap()
    );
}

#[test]
pub fn output_workflow_skyward_template_1() {
    let (wf, fncalls, metadata) = workflow_skyward_template_data_1();

    println!(
        "------------------------------ WORKFLOW SKYWARD ------------------------------\n{}",
        serde_json::to_string(&wf).unwrap()
    );

    println!(
            "------------------------------ WORKFLOW SKYWARD FNCALLS ------------------------------\n{}",
            serde_json::to_string(&fncalls).unwrap()
        );

    println!(
            "------------------------------ WORKFLOW SKYWARD FN_METADATA ------------------------------\n{}",
            serde_json::to_string(&metadata).unwrap()
        );
}

#[test]
fn output_workflow_skyward_settings_1() {
    let (wfs, settings) = workflow_skyward_template_settings_data_1();

    println!(
            "------------------------------ TEMPLATE SETTINGS SKYWARD ------------------------------\n{}",
            serde_json::to_string(&wfs).unwrap()
        );

    println!(
            "------------------------------ PROPOSE SETTINGS SKYWARD ------------------------------\n{}",
            serde_json::to_string(&settings).unwrap()
        );
}

#[test]
pub fn output_workflow_bounty_template_1() {
    let (wf, fncalls, metadata) = workflow_bounty_template_data_1();

    println!(
        "------------------------------ WORKFLOW BOUNTY ------------------------------\n{}",
        serde_json::to_string(&wf).unwrap()
    );
}

#[test]
fn output_workflow_bounty_settings_1() {
    let (wfs, settings) = workflow_bounty_template_settings_data_1();

    println!(
            "------------------------------ TEMPLATE SETTINGS BOUNTY  ------------------------------\n{}",
            serde_json::to_string(&wfs).unwrap()
        );

    println!(
            "------------------------------ PROPOSE SETTINGS BOUNTY  ------------------------------\n{}",
            serde_json::to_string(&settings).unwrap()
        );
}

#[test]
fn output_settings() {
    println!(
            "------------------------------ TEMPLATE SETTINGS ADD WORFLOW ------------------------------\n{}",
            serde_json::to_string(&workflow_settings_wf_add()).unwrap()
        );

    let wf_settings_add_wf = ProposeSettings {
        binds: vec![DataType::U16(2)],
        storage_key: "wf_add_wf_1".into(),
    };

    println!(
            "------------------------------ PROPOSE SETTINGS ADD WORKFLOW ------------------------------\n{}",
            serde_json::to_string(&wf_settings_add_wf).unwrap()
        );

    println!(
            "------------------------------ TEMPLATE SETTINGS BASIC WORKFLOW ------------------------------\n{}",
            serde_json::to_string(&workflow_settings_basic()).unwrap()
        );

    println!(
            "------------------------------ TEMPLATE SETTINGS SEND NEAR IN LOOP WORKFLOW ------------------------------\n{}",
            serde_json::to_string(&workflow_settings_treasury_send_near_loop()).unwrap()
        );

    let wf_settings_near_send = ProposeSettings {
        binds: vec![DataType::U128(10u128.pow(24).into())],
        storage_key: "wf_send_near_1".into(),
    };

    println!(
            "------------------------------ PROPOSE SETTINGS SEND NEAR IN LOOP WORKFLOW ------------------------------\n{}",
            serde_json::to_string(&wf_settings_near_send).unwrap()
        );
}
 */

#[test]
fn standard_fn_calls() {
    pretty_print("NEP_141_FT_TRANSFER", nep_141_ft_transfer());
    pretty_print("NEP_141_FT_TRANSFER_CALL", nep_141_ft_transfer_call());
    pretty_print("NEP_171_NFT_TRANSFER", nep_171_nft_transfer());
    pretty_print("NEP_171_NFT_TRANSFER_CALL", nep_171_nft_transfer_call());
}
