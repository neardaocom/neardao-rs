use std::collections::HashMap;

use near_sdk::{AccountId, ONE_NEAR, ONE_YOCTO};

use library::{
    types::{
        datatype::{Datatype, Value},
        source::SourceDataVariant,
    },
    workflow::{
        action::{
            ActionData, DaoActionData, FnCallData, FnCallIdType, InputSource, TemplateAction,
        },
        activity::{Activity, TemplateActivity, Terminality, Transition, TransitionLimit},
        postprocessing::Postprocessing,
        settings::{ActivityBind, ProposeSettings, TemplateSettings},
        template::Template,
        types::{
            ActivityRight, DaoActionIdent, Instruction, ObjectMetadata, Src,
            ValueSrc, VoteScenario,
        },
    },
};

use crate::TemplateData;

pub const DEFAULT_VOTING_DURATION: u32 = 10;

pub struct WfBasicPkg1ProposeOptions {
    pub template_id: u16,
    pub provider_id: String,
}

pub const WF_BASIC_PKG1_PROVIDER_ID_KEY: &str = "provider_id";
pub const WF_BASIC_PKG1_TEMPLATE_ID_KEY: &str = "id";

pub const WF_BASIC_PKG1_SETTINGS_DEPOSIT_PROPOSE: u128 = ONE_NEAR;
pub const WF_BASIC_PKG1_SETTINGS_DEPOSIT_VOTE: u128 = ONE_YOCTO;

/// Basic workflow package.
pub struct WfBasicPkg1;
impl WfBasicPkg1 {
    pub fn template(provider_id: String) -> TemplateData {
        let provider_id = AccountId::try_from(provider_id).expect("invalid account_id string");
        let map = HashMap::new();
        let tpl = Template {
            code: "basic_pkg1".into(),
            version: "1".into(),
            auto_exec: true,
            need_storage: false, // TODO: Not sure if true is true.
            receiver_storage_keys: vec![],
            activities: vec![
                Activity::Init,
                Activity::Activity(TemplateActivity {
                    code: "wf_add".into(),
                    postprocessing: None,
                    actions: vec![TemplateAction {
                        exec_condition: None,
                        validators: vec![],
                        action_data: ActionData::FnCall(FnCallData {
                            id: FnCallIdType::Dynamic(
                                ValueSrc::Src(Src::Input(
                                    WF_BASIC_PKG1_PROVIDER_ID_KEY.into(),
                                )),
                                "wf_template".into(),
                            ),
                            tgas: 30,
                            deposit: None,
                            binds: vec![],
                            must_succeed: true,
                        }),
                        postprocessing: Some(Postprocessing {
                            instructions: vec![Instruction::StoreWorkflow],
                        }),
                        optional: false,
                        input_source: InputSource::PropSettings,
                    }],
                    automatic: true,
                    terminal: Terminality::Automatic,
                    is_sync: false,
                }),
                Activity::Activity(TemplateActivity {
                    code: "media_add".into(),
                    postprocessing: None,
                    actions: vec![TemplateAction {
                        exec_condition: None,
                        validators: vec![],
                        action_data: ActionData::Action(DaoActionData {
                            code: None,
                            expected_input: None,
                            required_deposit: None,
                            binds: vec![],
                            name: DaoActionIdent::MediaAdd,
                        }),
                        postprocessing: None,
                        optional: false,
                        input_source: InputSource::PropSettings,
                    }],
                    automatic: true,
                    terminal: Terminality::Automatic,
                    is_sync: true,
                }),
                Activity::Activity(TemplateActivity {
                    code: "near_send".into(),
                    postprocessing: None,
                    actions: vec![TemplateAction {
                        exec_condition: None,
                        validators: vec![],
                        action_data: ActionData::SendNear(
                            ValueSrc::Src(Src::Input("receiver_id".into())),
                            ValueSrc::Src(Src::Input("amount".into())),
                        ),
                        optional: false,
                        postprocessing: None,
                        input_source: InputSource::PropSettings,
                    }],
                    automatic: true,
                    terminal: Terminality::Automatic,
                    is_sync: false,
                }),
            ],
            expressions: vec![],
            transitions: vec![vec![
                Transition {
                    activity_id: 1,
                    cond: None,
                    time_from_cond: None,
                    time_to_cond: None,
                },
                Transition {
                    activity_id: 2,
                    cond: None,
                    time_from_cond: None,
                    time_to_cond: None,
                },
                Transition {
                    activity_id: 3,
                    cond: None,
                    time_from_cond: None,
                    time_to_cond: None,
                },
            ]],
            constants: SourceDataVariant::Map(map),
            end: vec![1, 2, 3],
        };
        let fn_calls = vec![(provider_id, "wf_template".to_string())];
        let metadata = vec![vec![ObjectMetadata {
            arg_names: vec!["id".into()],
            arg_types: vec![Datatype::U64(false)],
        }]];
        (tpl, fn_calls, metadata, vec![])
    }
    pub fn propose_settings(options: Option<WfBasicPkg1ProposeOptions>) -> ProposeSettings {
        let WfBasicPkg1ProposeOptions {
            template_id,
            provider_id,
        } = options.expect("WfBasicPkg1ProposeOptions default options are not supported yet");
        let mut wf_add_constants = HashMap::new();
        wf_add_constants.insert(
            WF_BASIC_PKG1_PROVIDER_ID_KEY.into(),
            Value::String(provider_id.clone()),
        );
        wf_add_constants.insert(
            WF_BASIC_PKG1_TEMPLATE_ID_KEY.into(),
            Value::U64(template_id as u64),
        );

        // User proposed settings type
        let settings = ProposeSettings {
            constants: None,
            activity_constants: vec![
                None,
                Some(ActivityBind {
                    constants: None,
                    actions_constants: vec![Some(SourceDataVariant::Map(wf_add_constants))],
                }),
                None,
                None,
            ],
            storage_key: None,
        };
        settings
    }

    /// Default template settings for workflow: wf_add.
    pub fn template_settings(duration: Option<u32>) -> TemplateSettings {
        TemplateSettings {
            allowed_proposers: vec![ActivityRight::Group(1)],
            allowed_voters: ActivityRight::Group(1),
            activity_rights: vec![
                vec![],
                vec![ActivityRight::Group(1)],
                vec![ActivityRight::Group(1)],
                vec![ActivityRight::Group(1)],
            ],
            transition_limits: vec![vec![
                TransitionLimit { to: 1, limit: 1 },
                TransitionLimit { to: 2, limit: 1 },
                TransitionLimit { to: 3, limit: 1 },
            ]],
            scenario: VoteScenario::Democratic,
            duration: duration.unwrap_or(DEFAULT_VOTING_DURATION),
            quorum: 51,
            approve_threshold: 20,
            spam_threshold: 80,
            vote_only_once: true,
            deposit_propose: Some(Self::deposit_propose().into()),
            deposit_vote: Some(Self::deposit_vote().into()),
            deposit_propose_return: 0,
            constants: None,
        }
    }
    pub fn deposit_propose() -> u128 {
        WF_BASIC_PKG1_SETTINGS_DEPOSIT_PROPOSE
    }
    pub fn deposit_vote() -> u128 {
        WF_BASIC_PKG1_SETTINGS_DEPOSIT_VOTE
    }
}