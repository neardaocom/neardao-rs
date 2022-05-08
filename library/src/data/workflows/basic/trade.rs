use std::collections::HashMap;

use near_sdk::{ONE_NEAR, ONE_YOCTO};

use crate::{
    data::TemplateData,
    interpreter::expression::{EExpr, EOp, ExprTerm, LogOp, Op, RelOp, TExpr},
    types::{datatype::Value, source::SourceDataVariant},
    workflow::{
        action::{ActionType, TemplateAction},
        activity::{Activity, TemplateActivity, Terminality, Transition, TransitionLimit},
        expression::Expression,
        settings::{ProposeSettings, TemplateSettings},
        template::Template,
        types::{ActivityRight, ArgSrc, VoteScenario},
    },
};

pub const DEFAULT_VOTING_DURATION: u32 = 10;

pub struct Trade1ProposeOptions {
    pub required_token_id: String,
    pub required_token_amount: u128,
    pub offered_near_amount: u128,
}

pub const TRADE1_STORAGE_KEY: &str = "storage_key_trade1";

pub const TRADE1_REQUIRED_TOKEN_KEY: &str = "required_token_id";
pub const TRADE1_REQUIRED_AMOUNT_KEY: &str = "required_token_amount";
pub const TRADE1_OFFERED_AMOUNT_KEY: &str = "offered_near_amount";

pub const TRADE1_SETTINGS_DEPOSIT_PROPOSE: u128 = ONE_NEAR;
pub const TRADE1_SETTINGS_DEPOSIT_VOTE: u128 = ONE_YOCTO;

/// Simple trade workflow. Send NEAR after receiving some amount of tokens.
pub struct Trade1;
impl Trade1 {
    pub fn template() -> TemplateData {
        let template = Template {
            code: "trade1".into(),
            version: "1".into(),
            auto_exec: false,
            need_storage: true,
            activities: vec![
                Activity::Init,
                Activity::Activity(TemplateActivity {
                    code: "send_near".into(),
                    postprocessing: None,
                    actions: vec![TemplateAction {
                        exec_condition: Some(Expression {
                            args: vec![
                                ArgSrc::ConstPropSettings(TRADE1_REQUIRED_TOKEN_KEY.into()),
                                ArgSrc::Storage("token_id".into()),
                                ArgSrc::ConstPropSettings(TRADE1_REQUIRED_AMOUNT_KEY.into()),
                                ArgSrc::Storage("amount".into()),
                            ],
                            expr_id: 0,
                        }),
                        validators: vec![],
                        action_data: ActionType::SendNear(
                            ArgSrc::Storage("sender_id".into()),
                            ArgSrc::ConstPropSettings(TRADE1_OFFERED_AMOUNT_KEY.into()),
                        ),
                        must_succeed: true,
                        optional: false,
                        postprocessing: None,
                    }],
                    automatic: true,
                    terminal: Terminality::Automatic,
                    is_sync: false,
                }),
            ],
            expressions: vec![EExpr::Boolean(TExpr {
                operators: vec![
                    Op {
                        operands_ids: [0, 1],
                        op_type: EOp::Rel(RelOp::Eqs),
                    },
                    Op {
                        operands_ids: [2, 3],
                        op_type: EOp::Rel(RelOp::Eqs),
                    },
                    Op {
                        operands_ids: [0, 1],
                        op_type: EOp::Log(LogOp::And),
                    },
                ],
                terms: vec![
                    ExprTerm::Arg(0),
                    ExprTerm::Arg(1),
                    ExprTerm::Arg(2),
                    ExprTerm::Arg(3),
                ],
            })],
            transitions: vec![vec![Transition {
                activity_id: 1,
                cond: None,
                time_from_cond: None,
                time_to_cond: None,
            }]],
            constants: SourceDataVariant::Map(HashMap::new()),
            end: vec![1],
        };

        (template, vec![], vec![])
    }
    pub fn propose_settings(
        options: Option<Trade1ProposeOptions>,
        storage_key: Option<&str>,
    ) -> ProposeSettings {
        let Trade1ProposeOptions {
            required_token_id,
            required_token_amount,
            offered_near_amount,
        } = options.expect("Trade1ProposeOptions default options are not supported yet");
        let mut global_consts = HashMap::new();
        global_consts.insert(
            TRADE1_REQUIRED_TOKEN_KEY.into(),
            Value::String(required_token_id),
        );
        global_consts.insert(
            TRADE1_REQUIRED_AMOUNT_KEY.into(),
            Value::U128(required_token_amount.into()),
        );
        global_consts.insert(
            TRADE1_OFFERED_AMOUNT_KEY.into(),
            Value::U128(offered_near_amount.into()),
        );

        // User proposed settings type
        let settings = ProposeSettings {
            global: Some(SourceDataVariant::Map(global_consts)),
            binds: vec![None, None],
            storage_key: Some(storage_key.unwrap_or(TRADE1_STORAGE_KEY).into()),
        };
        settings
    }

    /// Default testing template settings for workflow: wf_add.
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
        TRADE1_SETTINGS_DEPOSIT_PROPOSE
    }
    pub fn deposit_vote() -> u128 {
        TRADE1_SETTINGS_DEPOSIT_VOTE
    }
}
