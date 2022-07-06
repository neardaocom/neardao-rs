use std::collections::HashMap;

use crate::TemplateData;
use library::{
    types::Value,
    workflow::{
        action::{ActionData, DaoActionData, InputSource, TemplateAction},
        activity::{Activity, TemplateActivity, Terminality, Transition, TransitionLimit},
        settings::{ActivityBind, ProposeSettings, TemplateSettings},
        template::SourceDataVariant,
        template::Template,
        types::{ActivityRight, DaoActionIdent, VoteScenario},
    },
};
use near_sdk::{ONE_NEAR, ONE_YOCTO};

pub const DEFAULT_VOTING_DURATION: u32 = 10;

pub const GROUP1_SETTINGS_DEPOSIT_PROPOSE: u128 = ONE_NEAR;
pub const GROUP1_SETTINGS_DEPOSIT_VOTE: u128 = ONE_YOCTO;

/// TODO: Workflow description.
pub struct Group1;
impl Group1 {
    pub fn template() -> TemplateData {
        let template = Template {
            code: "group1".into(),
            version: "1".into(),
            auto_exec: false,
            need_storage: false,
            receiver_storage_keys: vec![],
            activities: vec![
                Activity::Init,
                Activity::Activity(TemplateActivity {
                    code: "group_add".into(),
                    postprocessing: None,
                    actions: vec![TemplateAction {
                        exec_condition: None,
                        validators: vec![],
                        action_data: ActionData::Action(DaoActionData {
                            code: None,
                            expected_input: None,
                            required_deposit: None,
                            binds: vec![],
                            name: DaoActionIdent::GroupAdd,
                        }),
                        postprocessing: None,
                        optional: false,
                        input_source: InputSource::PropSettings,
                    }],
                    automatic: false,
                    terminal: Terminality::Automatic,
                    is_sync: true,
                }),
                Activity::Activity(TemplateActivity {
                    code: "group_remove".into(),
                    postprocessing: None,
                    actions: vec![TemplateAction {
                        exec_condition: None,
                        validators: vec![],
                        action_data: ActionData::Action(DaoActionData {
                            code: None,
                            expected_input: None,
                            required_deposit: None,
                            binds: vec![],
                            name: DaoActionIdent::GroupRemove,
                        }),
                        postprocessing: None,
                        optional: false,
                        input_source: InputSource::PropSettings,
                    }],
                    automatic: false,
                    terminal: Terminality::Automatic,
                    is_sync: true,
                }),
                Activity::Activity(TemplateActivity {
                    code: "group_add_members".into(),
                    postprocessing: None,
                    actions: vec![TemplateAction {
                        exec_condition: None,
                        validators: vec![],
                        action_data: ActionData::Action(DaoActionData {
                            code: None,
                            expected_input: None,
                            required_deposit: None,
                            binds: vec![],
                            name: DaoActionIdent::GroupAddMembers,
                        }),
                        postprocessing: None,
                        optional: false,
                        input_source: InputSource::PropSettings,
                    }],
                    automatic: false,
                    terminal: Terminality::Automatic,
                    is_sync: true,
                }),
                Activity::Activity(TemplateActivity {
                    code: "group_remove_members".into(),
                    postprocessing: None,
                    actions: vec![TemplateAction {
                        exec_condition: None,
                        validators: vec![],
                        action_data: ActionData::Action(DaoActionData {
                            code: None,
                            expected_input: None,
                            required_deposit: None,
                            binds: vec![],
                            name: DaoActionIdent::GroupRemoveMembers,
                        }),
                        postprocessing: None,
                        optional: false,
                        input_source: InputSource::PropSettings,
                    }],
                    automatic: false,
                    terminal: Terminality::Automatic,
                    is_sync: true,
                }),
                Activity::Activity(TemplateActivity {
                    code: "group_remove_roles".into(),
                    postprocessing: None,
                    actions: vec![TemplateAction {
                        exec_condition: None,
                        validators: vec![],
                        action_data: ActionData::Action(DaoActionData {
                            code: None,
                            expected_input: None,
                            required_deposit: None,
                            binds: vec![],
                            name: DaoActionIdent::GroupRemoveRoles,
                        }),
                        postprocessing: None,
                        optional: false,
                        input_source: InputSource::PropSettings,
                    }],
                    automatic: false,
                    terminal: Terminality::Automatic,
                    is_sync: true,
                }),
                Activity::Activity(TemplateActivity {
                    code: "group_remove_member_roles".into(),
                    postprocessing: None,
                    actions: vec![TemplateAction {
                        exec_condition: None,
                        validators: vec![],
                        action_data: ActionData::Action(DaoActionData {
                            code: None,
                            expected_input: None,
                            required_deposit: None,
                            binds: vec![],
                            name: DaoActionIdent::GroupRemoveMemberRoles,
                        }),
                        postprocessing: None,
                        optional: false,
                        input_source: InputSource::PropSettings,
                    }],
                    automatic: false,
                    terminal: Terminality::Automatic,
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
                    Transition {
                        activity_id: 3,
                        cond: None,
                        time_from_cond: None,
                        time_to_cond: None,
                    },
                    Transition {
                        activity_id: 4,
                        cond: None,
                        time_from_cond: None,
                        time_to_cond: None,
                    },
                    Transition {
                        activity_id: 5,
                        cond: None,
                        time_from_cond: None,
                        time_to_cond: None,
                    },
                    Transition {
                        activity_id: 6,
                        cond: None,
                        time_from_cond: None,
                        time_to_cond: None,
                    },
                ],
                // From 1.
                vec![],
                // From 3.
                vec![],
                // From 4.
                vec![],
                // From 5.
                vec![],
                // From 6.
                vec![],
            ],
            constants: SourceDataVariant::Map(HashMap::new()),
            end: vec![1, 2, 3, 4, 5, 6],
        };
        (template, vec![], vec![], vec![])
    }
    pub fn propose_settings(inputs: Vec<Option<HashMap<String, Value>>>) -> ProposeSettings {
        let mut activity_constants = vec![None];
        for i in 0..6usize {
            if let Some(v) = inputs.get(i) {
                if let Some(m) = v.clone() {
                    activity_constants.push(Some(ActivityBind {
                        constants: None,
                        actions_constants: vec![Some(SourceDataVariant::Map(m))],
                    }))
                } else {
                    activity_constants.push(None);
                }
            } else {
                activity_constants.push(None);
            }
        }

        let settings = ProposeSettings {
            constants: None,
            activity_constants,
            storage_key: None,
        };
        settings
    }

    pub fn template_settings(duration: Option<u32>) -> TemplateSettings {
        TemplateSettings {
            allowed_proposers: vec![ActivityRight::Group(1)],
            allowed_voters: ActivityRight::Group(1),
            activity_rights: vec![
                vec![],
                vec![
                    ActivityRight::Group(1),
                    ActivityRight::Group(1),
                    ActivityRight::Group(1),
                    ActivityRight::Group(1),
                    ActivityRight::Group(1),
                    ActivityRight::Group(1),
                ],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
            ],
            transition_limits: vec![
                vec![
                    TransitionLimit { to: 1, limit: 1 },
                    TransitionLimit { to: 2, limit: 1 },
                    TransitionLimit { to: 3, limit: 1 },
                    TransitionLimit { to: 4, limit: 1 },
                    TransitionLimit { to: 5, limit: 1 },
                    TransitionLimit { to: 6, limit: 1 },
                ],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
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
        GROUP1_SETTINGS_DEPOSIT_PROPOSE
    }
    pub fn deposit_vote() -> u128 {
        GROUP1_SETTINGS_DEPOSIT_VOTE
    }
}
