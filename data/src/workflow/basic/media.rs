use std::collections::HashMap;

use crate::TemplateData;
use library::{
    types::{datatype::Value, source::SourceDataVariant},
    workflow::{
        action::{ActionData, DaoActionData, InputSource, TemplateAction},
        activity::{Activity, TemplateActivity, Terminality, Transition, TransitionLimit},
        settings::{ActivityBind, ProposeSettings, TemplateSettings},
        template::Template,
        types::{ActivityRight, DaoActionIdent, VoteScenario},
    },
};
use near_sdk::{ONE_NEAR, ONE_YOCTO};

pub const DEFAULT_VOTING_DURATION: u32 = 10;

pub const MEDIA1_SETTINGS_DEPOSIT_PROPOSE: u128 = ONE_NEAR;
pub const MEDIA1_SETTINGS_DEPOSIT_VOTE: u128 = ONE_YOCTO;

pub struct Media1;
impl Media1 {
    pub fn template() -> TemplateData {
        let template = Template {
            code: "media1".into(),
            version: "1".into(),
            auto_exec: false,
            need_storage: false,
            activities: vec![
                Activity::Init,
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
                    automatic: false,
                    terminal: Terminality::User,
                    is_sync: true,
                }),
                Activity::Activity(TemplateActivity {
                    code: "media_update".into(),
                    postprocessing: None,
                    actions: vec![TemplateAction {
                        exec_condition: None,
                        validators: vec![],
                        action_data: ActionData::Action(DaoActionData {
                            code: None,
                            expected_input: None,
                            required_deposit: None,
                            binds: vec![],
                            name: DaoActionIdent::MediaUpdate,
                        }),
                        postprocessing: None,
                        optional: false,
                        input_source: InputSource::PropSettings,
                    }],
                    automatic: false,
                    terminal: Terminality::User,
                    is_sync: true,
                }),
            ],
            expressions: vec![],
            transitions: vec![
                // From 0.
                vec![
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
                ],
                // From 1.
                vec![Transition {
                    activity_id: 2,
                    cond: None,
                    time_from_cond: None,
                    time_to_cond: None,
                }],
                // From 2.
                vec![],
            ],
            constants: SourceDataVariant::Map(HashMap::new()),
            end: vec![1, 2],
        };
        (template, vec![], vec![], vec![])
    }
    pub fn propose_settings(
        storage_key: Option<&str>,
        inputs_activity_1: HashMap<String, Value>,
        inputs_activity_2: HashMap<String, Value>,
    ) -> ProposeSettings {
        let settings = ProposeSettings {
            constants: None,
            activity_constants: vec![
                None,
                Some(ActivityBind {
                    constants: None,
                    actions_constants: vec![Some(SourceDataVariant::Map(inputs_activity_1))],
                }),
                Some(ActivityBind {
                    constants: None,
                    actions_constants: vec![Some(SourceDataVariant::Map(inputs_activity_2))],
                }),
            ],
            storage_key: storage_key.map(|k| k.to_string()),
        };
        settings
    }

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
                vec![
                    TransitionLimit { to: 1, limit: 1 },
                    TransitionLimit { to: 2, limit: 1 },
                ],
                vec![TransitionLimit { to: 2, limit: 1 }],
                vec![],
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
        MEDIA1_SETTINGS_DEPOSIT_PROPOSE
    }
    pub fn deposit_vote() -> u128 {
        MEDIA1_SETTINGS_DEPOSIT_VOTE
    }
}
