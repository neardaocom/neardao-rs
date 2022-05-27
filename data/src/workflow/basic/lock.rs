use std::collections::HashMap;

use near_sdk::{ONE_NEAR, ONE_YOCTO};

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

pub const DEFAULT_VOTING_DURATION: u32 = 10;

pub const LOCK1_SETTINGS_DEPOSIT_PROPOSE: u128 = ONE_NEAR;
pub const LOCK1_SETTINGS_DEPOSIT_VOTE: u128 = ONE_YOCTO;

/// Simple trade workflow. Send NEAR after receiving some amount of tokens.
pub struct Lock1;
impl Lock1 {
    pub fn template() -> TemplateData {
        let template = Template {
            code: "lock1".into(),
            version: "1".into(),
            auto_exec: false,
            need_storage: false,
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
                        input_source: InputSource::PropSettings,
                    }],
                    automatic: false,
                    terminal: Terminality::Automatic,
                    is_sync: true,
                }),
            ],
            expressions: vec![],
            transitions: vec![vec![Transition {
                activity_id: 1,
                cond: None,
                time_from_cond: None,
                time_to_cond: None,
            }]],
            constants: SourceDataVariant::Map(HashMap::new()),
            end: vec![1],
        };

        (template, vec![], vec![], vec![])
    }
    pub fn propose_settings(
        storage_key: Option<&str>,
        inputs_activity_1: HashMap<String, Value>,
    ) -> ProposeSettings {
        // User proposed settings type
        let settings = ProposeSettings {
            constants: None,
            activity_constants: vec![
                None,
                Some(ActivityBind {
                    constants: None,
                    actions_constants: vec![Some(SourceDataVariant::Map(inputs_activity_1))],
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
            activity_rights: vec![vec![], vec![ActivityRight::Group(1)]],
            transition_limits: vec![vec![TransitionLimit { to: 1, limit: 1 }]],
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
        LOCK1_SETTINGS_DEPOSIT_PROPOSE
    }
    pub fn deposit_vote() -> u128 {
        LOCK1_SETTINGS_DEPOSIT_VOTE
    }
}
