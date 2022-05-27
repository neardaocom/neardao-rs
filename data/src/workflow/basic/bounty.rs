use std::collections::HashMap;

use crate::TemplateData;
use library::{
    interpreter::expression::{EExpr, EOp, ExprTerm, LogOp, Op, RelOp, TExpr},
    types::{
        datatype::{Datatype, Value},
        source::SourceDataVariant,
    },
    workflow::{
        action::{ActionData, DaoActionData, InputSource, TemplateAction},
        activity::{Activity, TemplateActivity, Terminality, Transition, TransitionLimit},
        expression::Expression,
        postprocessing::Postprocessing,
        settings::{ProposeSettings, TemplateSettings},
        template::Template,
        types::{ActivityRight, DaoActionIdent, Instruction, Src, ValueSrc, VoteScenario},
        validator::{ObjectValidator, Validator},
    },
};
use near_sdk::{ONE_NEAR, ONE_YOCTO};

pub const DEFAULT_VOTING_DURATION: u32 = 10;

pub struct Bounty1ProposeOptions {
    pub max_offered_near_amount: u128,
}

pub const BOUNTY1_OFFERED_AMOUNT_KEY: &str = "max_offered_near_amount";
pub const BOUNTY1_STORAGE_KEY: &str = "storage_key_bounty1";

pub const BOUNTY1_SETTINGS_DEPOSIT_PROPOSE: u128 = ONE_NEAR;
pub const BOUNTY1_SETTINGS_DEPOSIT_VOTE: u128 = ONE_YOCTO;

/// Workflow description:
/// Dao makes bounty for certain task.
/// Anyone can apply. (0 -> 1)
/// Anyone can apply from unrealized task. (2 -> 1)
/// Dao then approves (1 -> 3) or dispaproves assigner. (1 -> 2)
/// Then one of following might happen:
/// - Assigner gives up (3 -> 2)
/// - Assigner marks it done and send result (3 -> 4)
/// - NOT IMPLEMENTED YET: Dao decides to cancel it because it takes too long. (3 -> 2) (in future this will be implemented using time contraints)
/// If assigner marks it done, then someone from dao evaluates result. (4 -> 5)
/// Another DAO member can send up to max offered amount of NEAR to the bounty hunter. (5 -> 6)
pub struct Bounty1;
impl Bounty1 {
    pub fn template() -> TemplateData {
        let template = Template {
            code: "bounty1".into(),
            version: "1".into(),
            auto_exec: false,
            need_storage: true,
            activities: vec![
                Activity::Init,
                Activity::Activity(TemplateActivity {
                    code: "event_checkin".into(),
                    postprocessing: None,
                    actions: vec![TemplateAction {
                        exec_condition: None,
                        validators: vec![],
                        action_data: ActionData::Action(DaoActionData {
                            code: Some("event_checkin".into()),
                            expected_input: None,
                            required_deposit: None,
                            binds: vec![],
                            name: DaoActionIdent::Event,
                        }),
                        postprocessing: Some(Postprocessing {
                            instructions: vec![Instruction::StoreDynValue(
                                "account_id_applied".into(),
                                ValueSrc::Src(Src::Runtime(2)),
                            )],
                        }),
                        optional: false,
                        input_source: InputSource::User,
                    }],
                    automatic: true,
                    terminal: Terminality::NonTerminal,
                    is_sync: true,
                }),
                Activity::Activity(TemplateActivity {
                    code: "event_unrealized".into(),
                    postprocessing: None,
                    actions: vec![TemplateAction {
                        exec_condition: None,
                        validators: vec![],
                        action_data: ActionData::Action(DaoActionData {
                            code: Some("event_unrealized".into()),
                            expected_input: None,
                            required_deposit: None,
                            binds: vec![],
                            name: DaoActionIdent::Event,
                        }),
                        postprocessing: Some(Postprocessing {
                            instructions: vec![Instruction::DeleteKey("account_id_applied".into())],
                        }),
                        optional: false,
                        input_source: InputSource::User,
                    }],
                    automatic: true,
                    terminal: Terminality::NonTerminal,
                    is_sync: true,
                }),
                Activity::Activity(TemplateActivity {
                    code: "event_approve".into(),
                    postprocessing: None,
                    actions: vec![TemplateAction {
                        exec_condition: None,
                        validators: vec![],
                        action_data: ActionData::Action(DaoActionData {
                            code: Some("event_approve".into()),
                            expected_input: None,
                            required_deposit: None,
                            binds: vec![],
                            name: DaoActionIdent::Event,
                        }),
                        postprocessing: Some(Postprocessing {
                            instructions: vec![
                                Instruction::StoreDynValue(
                                    "approved_by".into(),
                                    ValueSrc::Src(Src::Runtime(2)),
                                ),
                                Instruction::StoreDynValue(
                                    "checkin_accepted".into(),
                                    ValueSrc::Src(Src::User("checkin_accepted".into())),
                                ),
                            ],
                        }),
                        optional: false,
                        input_source: InputSource::User,
                    }],
                    automatic: true,
                    terminal: Terminality::NonTerminal,
                    is_sync: true,
                }),
                Activity::Activity(TemplateActivity {
                    code: "event_done".into(),
                    postprocessing: None,
                    actions: vec![TemplateAction {
                        exec_condition: None,
                        validators: vec![],
                        action_data: ActionData::Action(DaoActionData {
                            code: Some("event_done".into()),
                            expected_input: Some(vec![("result".into(), Datatype::String(false))]),
                            required_deposit: None,
                            binds: vec![],
                            name: DaoActionIdent::Event,
                        }),
                        postprocessing: Some(Postprocessing {
                            instructions: vec![Instruction::StoreDynValue(
                                "event_done_result".into(),
                                ValueSrc::Src(Src::User("result".into())),
                            )],
                        }),
                        optional: false,
                        input_source: InputSource::User,
                    }],
                    automatic: true,
                    terminal: Terminality::NonTerminal,
                    is_sync: true,
                }),
                Activity::Activity(TemplateActivity {
                    code: "event_done_approve".into(),
                    postprocessing: None,
                    actions: vec![TemplateAction {
                        exec_condition: None,
                        validators: vec![],
                        action_data: ActionData::Action(DaoActionData {
                            code: Some("event_done_approve".into()),
                            expected_input: Some(vec![(
                                "result_evaluation".into(),
                                Datatype::String(false),
                            )]),
                            required_deposit: None,
                            binds: vec![],
                            name: DaoActionIdent::Event,
                        }),
                        postprocessing: Some(Postprocessing {
                            instructions: vec![
                                Instruction::StoreDynValue(
                                    "event_done_approved_by".into(),
                                    ValueSrc::Src(Src::Runtime(2)),
                                ),
                                Instruction::StoreDynValue(
                                    "event_done_result_evaluation".into(),
                                    ValueSrc::Src(Src::User("result_evaluation".into())),
                                ),
                            ],
                        }),
                        optional: false,
                        input_source: InputSource::User,
                    }],
                    automatic: true,
                    terminal: Terminality::NonTerminal,
                    is_sync: true,
                }),
                Activity::Activity(TemplateActivity {
                    code: "send_near".into(),
                    postprocessing: None,
                    actions: vec![TemplateAction {
                        exec_condition: None,
                        validators: vec![
                            Validator::Object(ObjectValidator {
                                expression_id: 0,
                                value: vec![
                                    ValueSrc::Src(Src::User("amount_near".into())),
                                    ValueSrc::Src(Src::PropSettings(
                                        BOUNTY1_OFFERED_AMOUNT_KEY.into(),
                                    )),
                                ],
                            }),
                            Validator::Object(ObjectValidator {
                                expression_id: 1,
                                value: vec![
                                    ValueSrc::Src(Src::User("receiver_id".into())),
                                    ValueSrc::Src(Src::Storage("account_id_applied".into())),
                                ],
                            }),
                        ],
                        action_data: ActionData::SendNear(
                            ValueSrc::Src(Src::Storage("account_id_applied".into())),
                            ValueSrc::Src(Src::User("amount_near".into())),
                        ),
                        optional: false,
                        postprocessing: None, // Could be stored amount of sent NEARs.
                        input_source: InputSource::User,
                    }],
                    automatic: true,
                    terminal: Terminality::Automatic,
                    is_sync: false,
                }),
            ],
            expressions: vec![
                // Validator: Last activity: inputed amount <= max offered amount.
                EExpr::Boolean(TExpr {
                    operators: vec![Op {
                        op_type: EOp::Rel(RelOp::GtE),
                        operands_ids: [0, 1],
                    }],
                    terms: vec![ExprTerm::Arg(0), ExprTerm::Arg(1)],
                }),
                // 1) Validator: Last activity: inputer account_id == bounty_hunter account id.
                // 2) Transition condition - from 3 to 2: Bounty hunter decided to give up after approve.
                EExpr::Boolean(TExpr {
                    operators: vec![Op {
                        op_type: EOp::Rel(RelOp::Eqs),
                        operands_ids: [0, 1],
                    }],
                    terms: vec![ExprTerm::Arg(0), ExprTerm::Arg(1)],
                }),
                // 1) Transition condition - from 3 to 4: User application accepted.
                EExpr::Boolean(TExpr {
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
                        ExprTerm::Value(Value::Bool(true)),
                        ExprTerm::Arg(1),
                        ExprTerm::Arg(2),
                    ],
                }),
                // Transition condition - from 3 to 1: User application not accepted.
                EExpr::Boolean(TExpr {
                    operators: vec![Op {
                        op_type: EOp::Rel(RelOp::Eqs),
                        operands_ids: [0, 1],
                    }],
                    terms: vec![ExprTerm::Arg(0), ExprTerm::Value(Value::Bool(false))],
                }),
                EExpr::Boolean(TExpr {
                    operators: vec![Op {
                        op_type: EOp::Rel(RelOp::Eqs),
                        operands_ids: [0, 1],
                    }],
                    terms: vec![ExprTerm::Arg(0), ExprTerm::Arg(1)],
                }),
            ],
            transitions: vec![
                // From 0.
                vec![Transition {
                    activity_id: 1,
                    cond: None,
                    time_from_cond: None,
                    time_to_cond: None,
                }],
                // From 1.
                vec![
                    // When DAO member decides if to accept bounty hunter.
                    Transition {
                        activity_id: 3,
                        cond: None,
                        time_from_cond: None,
                        time_to_cond: None,
                    },
                ],
                // From 2.
                vec![
                    // Unrealized, anyone can apply again.
                    Transition {
                        activity_id: 1,
                        cond: None,
                        time_from_cond: None,
                        time_to_cond: None,
                    },
                ],
                // From 3.
                vec![
                    // Checkin not accepted and new bounty hunter applies.
                    Transition {
                        activity_id: 1,
                        cond: Some(ValueSrc::Expr(Expression {
                            args: vec![Src::Storage("checkin_accepted".into())],
                            expr_id: 3,
                        })),
                        time_from_cond: None,
                        time_to_cond: None,
                    },
                    // Bouny hunter decides to give up.
                    Transition {
                        activity_id: 2,
                        cond: Some(ValueSrc::Expr(Expression {
                            args: vec![Src::Storage("account_id_applied".into()), Src::Runtime(2)],
                            expr_id: 1,
                        })),
                        time_from_cond: None,
                        time_to_cond: None,
                    },
                    // Basically anyone can make it but only the bounty hunter gets the reward.
                    // This condition is "DOS" protection.
                    Transition {
                        activity_id: 4,
                        cond: Some(ValueSrc::Expr(Expression {
                            args: vec![
                                Src::Storage("checkin_accepted".into()),
                                Src::Storage("account_id_applied".into()),
                                Src::Runtime(2),
                            ],
                            expr_id: 2,
                        })),
                        time_from_cond: None,
                        time_to_cond: None,
                    },
                ],
                // From 4.
                vec![Transition {
                    activity_id: 5,
                    cond: None,
                    time_from_cond: None,
                    time_to_cond: None,
                }],
                // From 5.
                vec![Transition {
                    activity_id: 6,
                    cond: None,
                    time_from_cond: None,
                    time_to_cond: None,
                }],
            ],
            constants: SourceDataVariant::Map(HashMap::new()),
            end: vec![6],
        };
        (template, vec![], vec![], vec![])
    }
    pub fn propose_settings(
        options: Option<Bounty1ProposeOptions>,
        storage_key: Option<&str>,
    ) -> ProposeSettings {
        let Bounty1ProposeOptions {
            max_offered_near_amount,
        } = options.expect("Bounty1ProposeOptions default options are not supported yet");
        let mut global_consts = HashMap::new();
        global_consts.insert(
            BOUNTY1_OFFERED_AMOUNT_KEY.into(),
            Value::U128(max_offered_near_amount.into()),
        );

        // User proposed settings type
        let settings = ProposeSettings {
            constants: Some(SourceDataVariant::Map(global_consts)),
            activity_constants: vec![None, None, None, None, None, None, None],
            storage_key: Some(storage_key.unwrap_or(BOUNTY1_STORAGE_KEY).into()),
        };
        settings
    }

    pub fn template_settings(duration: Option<u32>) -> TemplateSettings {
        TemplateSettings {
            allowed_proposers: vec![ActivityRight::Group(1)],
            allowed_voters: ActivityRight::Group(1),
            activity_rights: vec![
                vec![],
                vec![ActivityRight::Anyone],
                vec![ActivityRight::Anyone],
                vec![ActivityRight::Group(1)],
                vec![ActivityRight::Anyone],
                vec![ActivityRight::Group(1)],
                vec![ActivityRight::GroupLeader(1)],
            ],
            transition_limits: vec![
                vec![TransitionLimit { to: 1, limit: 5 }],
                vec![
                    //TransitionLimit { to: 2, limit: 5 },
                    TransitionLimit { to: 3, limit: 5 },
                ],
                vec![TransitionLimit { to: 4, limit: 1 }],
                vec![
                    TransitionLimit { to: 1, limit: 1 },
                    TransitionLimit { to: 2, limit: 1 },
                    TransitionLimit { to: 4, limit: 1 },
                ],
                vec![TransitionLimit { to: 5, limit: 1 }],
                vec![TransitionLimit { to: 6, limit: 1 }],
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
        BOUNTY1_SETTINGS_DEPOSIT_PROPOSE
    }
    pub fn deposit_vote() -> u128 {
        BOUNTY1_SETTINGS_DEPOSIT_VOTE
    }
}
/*
//OLD VERSION
// This wf does not return bounty + deposit
pub fn workflow_bounty_template_data_1() -> TemplateData {
    let wf = Template {
        code: "wf_bounty".into(),
        version: 1,
        activities: vec![
            None,
            Some(TemplateActivity {
                code: "event_checkin".into(),
                exec_condition: None,
                action: ActionType::Event,
                action_data: Some(ActionData::Event(EventData {
                    code: "checkin".into(),
                    values: vec![DataTypeDef::String(false)],
                    deposit_from_bind: Some(1),
                })),
                arg_types: vec![],
                activity_inputs: vec![vec![ArgSrc::Free]],
                postprocessing: Some(Postprocessing {
                    storage_key: "pp_1".into(),
                    op_type: PostprocessingType::SaveUserValue((0, 0)),
                    instructions: vec![],
                }),
                must_succeed: true,
            }),
            Some(TemplateActivity {
                code: "event_unrealized".into(),
                exec_condition: None,
                action: ActionType::Event,
                action_data: Some(ActionData::Event(EventData {
                    code: "unrealized".into(),
                    values: vec![DataTypeDef::String(false)],
                    deposit_from_bind: None,
                })),
                arg_types: vec![],
                activity_inputs: vec![vec![ArgSrc::Free]],
                postprocessing: Some(Postprocessing {
                    storage_key: "pp_2".into(),
                    op_type: PostprocessingType::RemoveActionStorage("pp_1".into()),
                    instructions: vec![],
                }),
                must_succeed: true,
            }),
            Some(TemplateActivity {
                code: "event_approve".into(),
                exec_condition: None,
                action: ActionType::Event,
                action_data: Some(ActionData::Event(EventData {
                    code: "approve".into(),
                    values: vec![DataTypeDef::String(false), DataTypeDef::Bool(false)],
                    deposit_from_bind: None,
                })),
                arg_types: vec![],
                activity_inputs: vec![vec![ArgSrc::Free, ArgSrc::Free]],
                postprocessing: Some(Postprocessing {
                    storage_key: "pp_3".into(),
                    op_type: PostprocessingType::SaveUserValue((0, 1)),
                    instructions: vec![],
                }),
                must_succeed: true,
            }),
            Some(TemplateActivity {
                code: "event_done".into(),
                exec_condition: None,
                action: ActionType::Event,
                action_data: Some(ActionData::Event(EventData {
                    code: "done".into(),
                    values: vec![DataTypeDef::String(false), DataTypeDef::String(false)],
                    deposit_from_bind: None,
                })),
                arg_types: vec![],
                activity_inputs: vec![vec![ArgSrc::Free, ArgSrc::Free]],
                postprocessing: Some(Postprocessing {
                    storage_key: "pp_4".into(),
                    op_type: PostprocessingType::SaveUserValue((0, 1)),
                    instructions: vec![],
                }),
                must_succeed: true,
            }),
            Some(TemplateActivity {
                code: "event_done_approve".into(),
                exec_condition: None,
                action: ActionType::Event,
                action_data: Some(ActionData::Event(EventData {
                    code: "done_approve".into(),
                    values: vec![DataTypeDef::String(false), DataTypeDef::String(false)],
                    deposit_from_bind: None,
                })),
                arg_types: vec![],
                activity_inputs: vec![vec![ArgSrc::Free, ArgSrc::Free]],
                postprocessing: Some(Postprocessing {
                    storage_key: "pp_5".into(),
                    op_type: PostprocessingType::SaveUserValue((0, 1)),
                    instructions: vec![],
                }),
                must_succeed: true,
            }),
            Some(TemplateActivity {
                code: "send_near".into(),
                exec_condition: None,
                action: ActionType::TreasurySendNear,
                action_data: None,
                arg_types: vec![],
                activity_inputs: vec![vec![ArgSrc::Storage("pp_1".into()), ArgSrc::Free]],
                postprocessing: None,
                must_succeed: true,
            }),
        ],
        obj_validators: vec![
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![ValidatorType::Simple(0), ValidatorType::Simple(0)],
        ],
        validator_exprs: vec![
            Expression {
                args: vec![ExprArg::Bind(0), ExprArg::User(1)],
                expr: EExpr::Boolean(TExpr {
                    operators: vec![Op {
                        op_type: EOp::Rel(RelOp::GtE),
                        operands_ids: [0, 1],
                    }],
                    terms: vec![ExprTerm::Arg(0), ExprTerm::Arg(1)],
                }),
            },
            Expression {
                args: vec![ExprArg::Storage("pp_1".into()), ExprArg::User(0)],
                expr: EExpr::Boolean(TExpr {
                    operators: vec![Op {
                        op_type: EOp::Rel(RelOp::Eqs),
                        operands_ids: [0, 1],
                    }],
                    terms: vec![ExprTerm::Arg(0), ExprTerm::Arg(1)],
                }),
            },
        ],
        transitions: vec![
            vec![1],
            vec![2, 3],
            vec![1],
            vec![1, 2, 4],
            vec![5],
            vec![6],
        ],
        binds: vec![],
        start: vec![0],
        end: vec![6],
    };

    let metadata = vec![];
    let fncalls = vec![];

    (wf, fncalls, metadata)
}

pub fn workflow_bounty_template_settings_data_1() -> TemplateUserSettings {
    let wfs = vec![TemplateSettings {
        activity_rights: vec![
            vec![ActivityRight::Anyone],
            vec![ActivityRight::Anyone],
            vec![ActivityRight::Group(1)],
            vec![ActivityRight::Group(1)],
            vec![ActivityRight::Group(1)],
            vec![ActivityRight::GroupLeader(1)],
        ],
        allowed_proposers: vec![ActivityRight::Group(1)],
        allowed_voters: ActivityRight::TokenHolder,
        scenario: VoteScenario::TokenWeighted,
        duration: 35,
        quorum: 51,
        approve_threshold: 20,
        spam_threshold: 80,
        vote_only_once: true,
        deposit_propose: Some(1.into()),
        deposit_vote: Some(1000.into()),
        deposit_propose_return: 0,
        transition_constraints: vec![
            vec![Transition {
                transition_limit: 1,
                cond: None,
            }],
            vec![
                Transition {
                    transition_limit: 5,
                    cond: None,
                },
                Transition {
                    transition_limit: 5,
                    cond: None,
                },
            ],
            vec![Transition {
                transition_limit: 4,
                cond: None,
            }],
            vec![
                Transition {
                    transition_limit: 4,
                    cond: Some(Expression {
                        args: vec![ExprArg::Storage("pp_3".into())],
                        expr: EExpr::Boolean(TExpr {
                            operators: vec![Op {
                                op_type: EOp::Rel(RelOp::Eqs),
                                operands_ids: [0, 1],
                            }],
                            terms: vec![ExprTerm::Arg(0), ExprTerm::Value(DataType::Bool(false))],
                        }),
                    }),
                },
                Transition {
                    transition_limit: 4,
                    cond: Some(Expression {
                        args: vec![ExprArg::Storage("pp_1".into()), ExprArg::User(0)],
                        expr: EExpr::Boolean(TExpr {
                            operators: vec![Op {
                                op_type: EOp::Rel(RelOp::Eqs),
                                operands_ids: [0, 1],
                            }],
                            terms: vec![ExprTerm::Arg(0), ExprTerm::Arg(1)],
                        }),
                    }),
                },
                Transition {
                    transition_limit: 1,
                    cond: Some(Expression {
                        args: vec![ExprArg::Storage("pp_3".into())],
                        expr: EExpr::Boolean(TExpr {
                            operators: vec![Op {
                                op_type: EOp::Rel(RelOp::Eqs),
                                operands_ids: [0, 1],
                            }],
                            terms: vec![ExprTerm::Arg(0), ExprTerm::Value(DataType::Bool(true))],
                        }),
                    }),
                },
            ],
            vec![Transition {
                transition_limit: 1,
                cond: None,
            }],
            vec![Transition {
                transition_limit: 1,
                cond: None,
            }],
        ],
    }];

    let propose_setting = ProposeSettings {
        binds: vec![
            DataType::U128((5 * ONE_NEAR).into()),
            DataType::U128(ONE_NEAR.into()),
        ],
        storage_key: "wf_bounty_1".into(),
    };

    (wfs, propose_setting)
}
 */
