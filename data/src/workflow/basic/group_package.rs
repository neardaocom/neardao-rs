use std::collections::HashMap;

use crate::TemplateData;
use library::{
    types::source::SourceDataVariant,
    workflow::{
        action::{ActionData, DaoActionData, InputSource, TemplateAction},
        activity::{Activity, TemplateActivity, Terminality, Transition, TransitionLimit},
        settings::{ProposeSettings, TemplateSettings},
        template::Template,
        types::{ActivityRight, DaoActionIdent, VoteScenario},
    },
};
use near_sdk::{ONE_NEAR, ONE_YOCTO};

pub const DEFAULT_VOTING_DURATION: u32 = 10;

pub const GROUP_PACKAGE1_SETTINGS_DEPOSIT_PROPOSE: u128 = ONE_NEAR;
pub const GROUP_PACKAGE1_SETTINGS_DEPOSIT_VOTE: u128 = ONE_YOCTO;

/// TODO: Workflow description.
pub struct GroupPackage1;
impl GroupPackage1 {
    pub fn template() -> TemplateData {
        let template = Template {
            code: "group_package1".into(),
            version: "1".into(),
            auto_exec: false,
            need_storage: false,
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
                        input_source: InputSource::User,
                    }],
                    automatic: false,
                    terminal: Terminality::User,
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
                        input_source: InputSource::User,
                    }],
                    automatic: false,
                    terminal: Terminality::User,
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
                        input_source: InputSource::User,
                    }],
                    automatic: false,
                    terminal: Terminality::User,
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
                        input_source: InputSource::User,
                    }],
                    automatic: false,
                    terminal: Terminality::User,
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
                        input_source: InputSource::User,
                    }],
                    automatic: false,
                    terminal: Terminality::User,
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
                        input_source: InputSource::User,
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
                // From 2.
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
                // From 3.
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
                // From 4.
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
                // From 5.
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
                // From 6.
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
            ],
            constants: SourceDataVariant::Map(HashMap::new()),
            end: vec![1, 2, 3, 4, 5, 6],
        };
        (template, vec![], vec![], vec![])
    }
    pub fn propose_settings() -> ProposeSettings {
        let settings = ProposeSettings {
            constants: None,
            activity_constants: vec![None, None, None, None, None, None, None],
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
                vec![
                    ActivityRight::Group(1),
                    ActivityRight::Group(1),
                    ActivityRight::Group(1),
                    ActivityRight::Group(1),
                    ActivityRight::Group(1),
                    ActivityRight::Group(1),
                ],
                vec![
                    ActivityRight::Group(1),
                    ActivityRight::Group(1),
                    ActivityRight::Group(1),
                    ActivityRight::Group(1),
                    ActivityRight::Group(1),
                    ActivityRight::Group(1),
                ],
                vec![
                    ActivityRight::Group(1),
                    ActivityRight::Group(1),
                    ActivityRight::Group(1),
                    ActivityRight::Group(1),
                    ActivityRight::Group(1),
                    ActivityRight::Group(1),
                ],
                vec![
                    ActivityRight::Group(1),
                    ActivityRight::Group(1),
                    ActivityRight::Group(1),
                    ActivityRight::Group(1),
                    ActivityRight::Group(1),
                    ActivityRight::Group(1),
                ],
                vec![
                    ActivityRight::Group(1),
                    ActivityRight::Group(1),
                    ActivityRight::Group(1),
                    ActivityRight::Group(1),
                    ActivityRight::Group(1),
                    ActivityRight::Group(1),
                ],
            ],
            transition_limits: vec![
                vec![
                    TransitionLimit { to: 1, limit: 10 },
                    TransitionLimit { to: 2, limit: 10 },
                    TransitionLimit { to: 3, limit: 10 },
                    TransitionLimit { to: 4, limit: 10 },
                    TransitionLimit { to: 5, limit: 10 },
                    TransitionLimit { to: 6, limit: 10 },
                ],
                vec![
                    TransitionLimit { to: 1, limit: 10 },
                    TransitionLimit { to: 2, limit: 10 },
                    TransitionLimit { to: 3, limit: 10 },
                    TransitionLimit { to: 4, limit: 10 },
                    TransitionLimit { to: 5, limit: 10 },
                    TransitionLimit { to: 6, limit: 10 },
                ],
                vec![
                    TransitionLimit { to: 1, limit: 10 },
                    TransitionLimit { to: 2, limit: 10 },
                    TransitionLimit { to: 3, limit: 10 },
                    TransitionLimit { to: 4, limit: 10 },
                    TransitionLimit { to: 5, limit: 10 },
                    TransitionLimit { to: 6, limit: 10 },
                ],
                vec![
                    TransitionLimit { to: 1, limit: 10 },
                    TransitionLimit { to: 2, limit: 10 },
                    TransitionLimit { to: 3, limit: 10 },
                    TransitionLimit { to: 4, limit: 10 },
                    TransitionLimit { to: 5, limit: 10 },
                    TransitionLimit { to: 6, limit: 10 },
                ],
                vec![
                    TransitionLimit { to: 1, limit: 10 },
                    TransitionLimit { to: 2, limit: 10 },
                    TransitionLimit { to: 3, limit: 10 },
                    TransitionLimit { to: 4, limit: 10 },
                    TransitionLimit { to: 5, limit: 10 },
                    TransitionLimit { to: 6, limit: 10 },
                ],
                vec![
                    TransitionLimit { to: 1, limit: 10 },
                    TransitionLimit { to: 2, limit: 10 },
                    TransitionLimit { to: 3, limit: 10 },
                    TransitionLimit { to: 4, limit: 10 },
                    TransitionLimit { to: 5, limit: 10 },
                    TransitionLimit { to: 6, limit: 10 },
                ],
                vec![
                    TransitionLimit { to: 1, limit: 10 },
                    TransitionLimit { to: 2, limit: 10 },
                    TransitionLimit { to: 3, limit: 10 },
                    TransitionLimit { to: 4, limit: 10 },
                    TransitionLimit { to: 5, limit: 10 },
                    TransitionLimit { to: 6, limit: 10 },
                ],
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
        GROUP_PACKAGE1_SETTINGS_DEPOSIT_PROPOSE
    }
    pub fn deposit_vote() -> u128 {
        GROUP_PACKAGE1_SETTINGS_DEPOSIT_VOTE
    }
}
