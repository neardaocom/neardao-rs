use std::collections::HashMap;

use data::TemplateData;
use near_sdk::{ONE_NEAR, ONE_YOCTO};

use library::{
    types::{datatype::Value, source::SourceDataVariant},
    workflow::{
        action::{ActionData, DaoActionData, InputSource, TemplateAction},
        activity::{Activity, TemplateActivity, Terminality, Transition, TransitionLimit},
        postprocessing::Postprocessing,
        settings::{ProposeSettings, TemplateSettings},
        template::Template,
        types::{ActivityRight, DaoActionIdent, Instruction, Src, ValueSrc, VoteScenario},
    },
};

pub const DEFAULT_VOTING_DURATION: u32 = 10;

pub const WF_ADD1_SETTINGS_DEPOSIT_PROPOSE: u128 = ONE_NEAR;
pub const WF_ADD1_SETTINGS_DEPOSIT_VOTE: u128 = ONE_YOCTO;

pub struct WfOptionalActions;
impl WfOptionalActions {
    pub fn template() -> TemplateData {
        let map = HashMap::new();
        let tpl = Template {
            code: "wf_optional_fn_calls".into(),
            version: "1".into(),
            auto_exec: false,
            need_storage: true,
            activities: vec![
                Activity::Init,
                Activity::Activity(TemplateActivity {
                    code: "optional_events".into(),
                    postprocessing: None,
                    actions: vec![
                        TemplateAction {
                            exec_condition: None,
                            validators: vec![],
                            action_data: ActionData::Action(DaoActionData {
                                name: DaoActionIdent::Event,
                                code: Some("event_1".into()),
                                expected_input: None,
                                required_deposit: None,
                                binds: vec![],
                            }),
                            postprocessing: Some(Postprocessing {
                                instructions: vec![Instruction::StoreValueGlobal(
                                    "pp_activity_1_action_0".into(),
                                    Value::Bool(true),
                                )],
                            }),
                            optional: false,
                            input_source: InputSource::User,
                        },
                        TemplateAction {
                            exec_condition: None,
                            validators: vec![],
                            action_data: ActionData::Action(DaoActionData {
                                name: DaoActionIdent::Event,
                                code: Some("event_2".into()),
                                expected_input: None,
                                required_deposit: None,
                                binds: vec![],
                            }),
                            postprocessing: Some(Postprocessing {
                                instructions: vec![Instruction::StoreValueGlobal(
                                    "pp_activity_1_action_1".into(),
                                    Value::Bool(true),
                                )],
                            }),
                            optional: true,
                            input_source: InputSource::User,
                        },
                        TemplateAction {
                            exec_condition: None,
                            validators: vec![],
                            action_data: ActionData::Action(DaoActionData {
                                name: DaoActionIdent::Event,
                                code: Some("event_3".into()),
                                expected_input: None,
                                required_deposit: None,
                                binds: vec![],
                            }),
                            postprocessing: Some(Postprocessing {
                                instructions: vec![Instruction::StoreValueGlobal(
                                    "pp_activity_1_action_2".into(),
                                    Value::Bool(true),
                                )],
                            }),
                            optional: true,
                            input_source: InputSource::User,
                        },
                        TemplateAction {
                            exec_condition: None,
                            validators: vec![],
                            action_data: ActionData::Action(DaoActionData {
                                name: DaoActionIdent::Event,
                                code: Some("event_4".into()),
                                expected_input: None,
                                required_deposit: None,
                                binds: vec![],
                            }),
                            postprocessing: Some(Postprocessing {
                                instructions: vec![Instruction::StoreValueGlobal(
                                    "pp_activity_1_action_3".into(),
                                    Value::Bool(true),
                                )],
                            }),
                            optional: false,
                            input_source: InputSource::User,
                        },
                    ],
                    automatic: false,
                    terminal: Terminality::NonTerminal,
                    is_sync: true,
                }),
                Activity::Activity(TemplateActivity {
                    code: "optional_fncalls".into(),
                    postprocessing: None,
                    actions: vec![
                        TemplateAction {
                            exec_condition: None,
                            validators: vec![],
                            action_data: ActionData::SendNear(
                                ValueSrc::Src(Src::Runtime(2)),
                                ValueSrc::Value(Value::U128((5 * ONE_NEAR).into())),
                            ),
                            postprocessing: Some(Postprocessing {
                                instructions: vec![Instruction::StoreValueGlobal(
                                    "pp_activity_2_action_0".into(),
                                    Value::Bool(true),
                                )],
                            }),
                            optional: false,
                            input_source: InputSource::User,
                        },
                        TemplateAction {
                            exec_condition: None,
                            validators: vec![],
                            action_data: ActionData::SendNear(
                                ValueSrc::Src(Src::Runtime(2)),
                                ValueSrc::Value(Value::U128((5 * ONE_NEAR).into())),
                            ),
                            postprocessing: Some(Postprocessing {
                                instructions: vec![Instruction::StoreValueGlobal(
                                    "pp_activity_2_action_1".into(),
                                    Value::Bool(true),
                                )],
                            }),
                            optional: true,
                            input_source: InputSource::User,
                        },
                        TemplateAction {
                            exec_condition: None,
                            validators: vec![],
                            action_data: ActionData::SendNear(
                                ValueSrc::Src(Src::Runtime(2)),
                                ValueSrc::Value(Value::U128((5 * ONE_NEAR).into())),
                            ),
                            postprocessing: Some(Postprocessing {
                                instructions: vec![Instruction::StoreValueGlobal(
                                    "pp_activity_2_action_2".into(),
                                    Value::Bool(true),
                                )],
                            }),
                            optional: true,
                            input_source: InputSource::User,
                        },
                        TemplateAction {
                            exec_condition: None,
                            validators: vec![],
                            action_data: ActionData::SendNear(
                                ValueSrc::Src(Src::Runtime(2)),
                                ValueSrc::Value(Value::U128((5 * ONE_NEAR).into())),
                            ),
                            postprocessing: Some(Postprocessing {
                                instructions: vec![Instruction::StoreValueGlobal(
                                    "pp_activity_2_action_3".into(),
                                    Value::Bool(true),
                                )],
                            }),
                            optional: false,
                            input_source: InputSource::User,
                        },
                    ],
                    automatic: false,
                    terminal: Terminality::NonTerminal,
                    is_sync: false,
                }),
            ],
            expressions: vec![],
            transitions: vec![
                vec![Transition {
                    activity_id: 1,
                    cond: None,
                    time_from_cond: None,
                    time_to_cond: None,
                }],
                vec![Transition {
                    activity_id: 2,
                    cond: None,
                    time_from_cond: None,
                    time_to_cond: None,
                }],
            ],
            constants: SourceDataVariant::Map(map),
            end: vec![2],
        };
        let fn_calls = vec![];
        let metadata = vec![];
        (tpl, fn_calls, metadata, vec![])
    }
    pub fn propose_settings(storage_key: &str) -> ProposeSettings {
        let settings = ProposeSettings {
            constants: None,
            activity_constants: vec![None, None, None],
            storage_key: Some(storage_key.into()),
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
            ],
            transition_limits: vec![
                vec![TransitionLimit { to: 1, limit: 1 }],
                vec![TransitionLimit { to: 2, limit: 1 }],
            ],
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
        WF_ADD1_SETTINGS_DEPOSIT_PROPOSE
    }
    pub fn deposit_vote() -> u128 {
        WF_ADD1_SETTINGS_DEPOSIT_VOTE
    }
}
