use std::collections::HashMap;

use near_sdk::{ONE_NEAR, ONE_YOCTO};

use crate::TemplateData;
use library::workflow::{
    action::{ActionData, DaoActionData, InputSource, TemplateAction},
    activity::{Activity, TemplateActivity, Terminality, Transition, TransitionLimit},
    settings::{ProposeSettings, TemplateSettings},
    template::SourceDataVariant,
    template::Template,
    types::{ActivityRight, DaoActionIdent, VoteScenario},
};

pub const DEFAULT_VOTING_DURATION: u32 = 10;

pub struct Reward1ProposeOptions {
    pub required_token_id: String,
    pub required_token_amount: u128,
    pub offered_near_amount: u128,
}

pub const REWARD1_STORAGE_KEY: &str = "storage_key_reward1";

pub const REWARD1_SETTINGS_DEPOSIT_PROPOSE: u128 = ONE_NEAR;
pub const REWARD1_SETTINGS_DEPOSIT_VOTE: u128 = ONE_YOCTO;

/// Simple trade workflow. Send NEAR after receiving some amount of tokens.
pub struct Reward1;
impl Reward1 {
    pub fn template() -> TemplateData {
        let template = Template {
            code: "reward1".into(),
            version: "1".into(),
            auto_exec: false,
            need_storage: false,
            receiver_storage_keys: vec![],
            activities: vec![
                Activity::Init,
                Activity::Activity(TemplateActivity {
                    code: "treasury_add_partition".into(),
                    postprocessing: None,
                    actions: vec![TemplateAction {
                        exec_condition: None,
                        validators: vec![],
                        action_data: ActionData::Action(DaoActionData {
                            name: DaoActionIdent::TreasuryAddPartition,
                            required_deposit: None,
                            binds: vec![],
                            code: None,
                            expected_input: None,
                        }),
                        optional: false,
                        postprocessing: None,
                        input_source: InputSource::User,
                    }],
                    automatic: false,
                    terminal: Terminality::NonTerminal,
                    is_sync: true,
                }),
                Activity::Activity(TemplateActivity {
                    code: "reward_add_wage".into(),
                    postprocessing: None,
                    actions: vec![TemplateAction {
                        exec_condition: None,
                        validators: vec![],
                        action_data: ActionData::Action(DaoActionData {
                            name: DaoActionIdent::RewardAdd,
                            required_deposit: None,
                            binds: vec![],
                            code: None,
                            expected_input: None,
                        }),
                        postprocessing: None,
                        optional: false,
                        input_source: InputSource::User,
                    }],
                    automatic: false,
                    terminal: Terminality::Automatic,
                    is_sync: true,
                }),
                Activity::Activity(TemplateActivity {
                    code: "reward_add_user_activity".into(),
                    postprocessing: None,
                    actions: vec![TemplateAction {
                        exec_condition: None,
                        validators: vec![],
                        action_data: ActionData::Action(DaoActionData {
                            name: DaoActionIdent::RewardAdd,
                            required_deposit: None,
                            binds: vec![],
                            code: None,
                            expected_input: None,
                        }),
                        optional: false,
                        postprocessing: None,
                        input_source: InputSource::User,
                    }],
                    automatic: false,
                    terminal: Terminality::Automatic,
                    is_sync: true,
                }),
            ],
            expressions: vec![],
            transitions: vec![
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
                ],
                vec![
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
                ],
            ],
            constants: SourceDataVariant::Map(HashMap::new()),
            end: vec![2, 3],
        };

        (template, vec![], vec![], vec![])
    }
    pub fn propose_settings(storage_key: Option<&str>) -> ProposeSettings {
        let settings = ProposeSettings {
            constants: None,
            activity_constants: vec![None, None, None, None],
            storage_key: Some(storage_key.unwrap_or(REWARD1_STORAGE_KEY).into()),
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
                vec![ActivityRight::Group(1)],
            ],
            transition_limits: vec![
                vec![
                    TransitionLimit { to: 1, limit: 1 },
                    TransitionLimit { to: 2, limit: 1 },
                    TransitionLimit { to: 3, limit: 1 },
                ],
                vec![
                    TransitionLimit { to: 2, limit: 1 },
                    TransitionLimit { to: 3, limit: 1 },
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
        REWARD1_SETTINGS_DEPOSIT_PROPOSE
    }
    pub fn deposit_vote() -> u128 {
        REWARD1_SETTINGS_DEPOSIT_VOTE
    }
}

pub struct Reward2;
impl Reward2 {
    pub fn template() -> TemplateData {
        let template = Template {
            code: "reward2".into(),
            version: "1".into(),
            auto_exec: false,
            need_storage: false,
            activities: vec![
                Activity::Init,
                Activity::Activity(TemplateActivity {
                    code: "reward_add_wage".into(),
                    postprocessing: None,
                    actions: vec![TemplateAction {
                        exec_condition: None,
                        validators: vec![],
                        action_data: ActionData::Action(DaoActionData {
                            name: DaoActionIdent::RewardAdd,
                            required_deposit: None,
                            binds: vec![],
                            code: None,
                            expected_input: None,
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
                    code: "reward_add_user_activity".into(),
                    postprocessing: None,
                    actions: vec![TemplateAction {
                        exec_condition: None,
                        validators: vec![],
                        action_data: ActionData::Action(DaoActionData {
                            name: DaoActionIdent::RewardAdd,
                            required_deposit: None,
                            binds: vec![],
                            code: None,
                            expected_input: None,
                        }),
                        optional: false,
                        postprocessing: None,
                        input_source: InputSource::PropSettings,
                    }],
                    automatic: false,
                    terminal: Terminality::Automatic,
                    is_sync: true,
                }),
                Activity::Activity(TemplateActivity {
                    code: "reward_update".into(),
                    postprocessing: None,
                    actions: vec![TemplateAction {
                        exec_condition: None,
                        validators: vec![],
                        action_data: ActionData::Action(DaoActionData {
                            name: DaoActionIdent::RewardUpdate,
                            required_deposit: None,
                            binds: vec![],
                            code: None,
                            expected_input: None,
                        }),
                        optional: false,
                        postprocessing: None,
                        input_source: InputSource::PropSettings,
                    }],
                    automatic: false,
                    terminal: Terminality::Automatic,
                    is_sync: true,
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
            constants: SourceDataVariant::Map(HashMap::new()),
            end: vec![1, 2, 3],
            receiver_storage_keys: vec![],
        };

        (template, vec![], vec![], vec![])
    }
    pub fn propose_settings(storage_key: Option<&str>) -> ProposeSettings {
        let settings = ProposeSettings {
            constants: None,
            activity_constants: vec![None, None, None, None],
            storage_key: Some(storage_key.unwrap_or(REWARD1_STORAGE_KEY).into()),
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
        REWARD1_SETTINGS_DEPOSIT_PROPOSE
    }
    pub fn deposit_vote() -> u128 {
        REWARD1_SETTINGS_DEPOSIT_VOTE
    }
}
