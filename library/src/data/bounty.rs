use std::vec;

use crate::{
    expression::{EExpr, EOp, ExprTerm, FnName, Op, RelOp, TExpr},
    types::{
        ActionData, ActionIdent, DataType, DataTypeDef, EventData, FnCallMetadata, ValidatorType,
    },
    unit_tests::ONE_NEAR,
    workflow::{
        ActivityRight, ArgType, CondOrExpr, ExprArg, Expression, Postprocessing,
        PostprocessingType, ProposeSettings, Template, TemplateActivity, TemplateSettings,
        TransitionConstraint, VoteScenario,
    },
    FnCallId,
};

use super::{TemplateData, TemplateUserSettings};

// Activities:
//
//  1. CheckIn (accountId) - save account to storage
//     Unrealized (accountId) - remove account from storage
//     Approve (accountId, State) - true: save else: remove account from storage // tcond = pp_1 exists // pp - true => save_value_true, pp - false => save_value_false // tcond = pp_3 = false
//     Done (accountId, note) - check account, // tcond = pp_3 = true, pp - save user value 2
//     Approve Done (accountId, state, note) tcond = pp4 exists
//     Payout (accountId, amount)
pub fn workflow_bounty_template_data_1() -> TemplateData {
    //TODO event deposit
    let wf = Template {
        name: "wf_bounty".into(),
        version: 1,
        activities: vec![
            None,
            Some(TemplateActivity {
                code: "event_checkin".into(),
                exec_condition: None,
                action: ActionIdent::Event,
                action_data: Some(ActionData::Event(EventData {
                    code: "checkin".into(),
                    values: vec![DataTypeDef::String(false)],
                    deposit_from_bind: Some(0),
                })),
                arg_types: vec![],
                activity_inputs: vec![vec![ArgType::Free]],
                postprocessing: Some(Postprocessing {
                    storage_key: "pp_1".into(),
                    op_type: PostprocessingType::SaveUserValue((0, 0)),
                    instructions: vec![],
                }),
            }),
            Some(TemplateActivity {
                code: "event_unrealized".into(),
                exec_condition: None,
                action: ActionIdent::Event,
                action_data: Some(ActionData::Event(EventData {
                    code: "unrealized".into(),
                    values: vec![DataTypeDef::String(false)],
                    deposit_from_bind: None,
                })),
                arg_types: vec![],
                activity_inputs: vec![vec![ArgType::Free]],
                postprocessing: Some(Postprocessing {
                    storage_key: "pp_2".into(),
                    op_type: PostprocessingType::RemoveActionStorage("pp_1".into()),
                    instructions: vec![],
                }),
            }),
            Some(TemplateActivity {
                code: "event_approve".into(),
                exec_condition: None,
                action: ActionIdent::Event,
                action_data: Some(ActionData::Event(EventData {
                    code: "approve".into(),
                    values: vec![DataTypeDef::String(false), DataTypeDef::Bool(false)],
                    deposit_from_bind: None,
                })),
                arg_types: vec![],
                activity_inputs: vec![vec![ArgType::Free, ArgType::Free]],
                postprocessing: Some(Postprocessing {
                    storage_key: "pp_3".into(),
                    op_type: PostprocessingType::SaveUserValue((0, 1)),
                    instructions: vec![],
                }),
            }),
            Some(TemplateActivity {
                code: "event_done".into(),
                exec_condition: None,
                action: ActionIdent::Event,
                action_data: Some(ActionData::Event(EventData {
                    code: "done".into(),
                    values: vec![DataTypeDef::String(false), DataTypeDef::String(false)],
                    deposit_from_bind: None,
                })),
                arg_types: vec![],
                activity_inputs: vec![vec![ArgType::Free, ArgType::Free]],
                postprocessing: Some(Postprocessing {
                    storage_key: "pp_4".into(),
                    op_type: PostprocessingType::SaveUserValue((0, 1)),
                    instructions: vec![],
                }),
            }),
            Some(TemplateActivity {
                code: "event_done_approve".into(),
                exec_condition: None,
                action: ActionIdent::Event,
                action_data: Some(ActionData::Event(EventData {
                    code: "done_approve".into(),
                    values: vec![DataTypeDef::String(false), DataTypeDef::Bool(false)],
                    deposit_from_bind: None,
                })),
                arg_types: vec![],
                activity_inputs: vec![vec![ArgType::Free, ArgType::Free]],
                postprocessing: Some(Postprocessing {
                    storage_key: "pp_5".into(),
                    op_type: PostprocessingType::SaveUserValue((0, 1)),
                    instructions: vec![],
                }),
            }),
            Some(TemplateActivity {
                code: "send_near".into(),
                exec_condition: None,
                action: ActionIdent::TreasurySendNear,
                action_data: None,
                arg_types: vec![],
                activity_inputs: vec![vec![ArgType::Free, ArgType::Free]],
                postprocessing: None,
            }),
        ],
        obj_validators: vec![vec![ValidatorType::Primitive(0)]],
        validator_exprs: vec![Expression {
            args: vec![ExprArg::Bind(0), ExprArg::User(1)],
            expr: EExpr::Boolean(TExpr {
                operators: vec![Op {
                    op_type: EOp::Rel(RelOp::GtE),
                    operands_ids: [0, 1],
                }],
                terms: vec![ExprTerm::Arg(0), ExprTerm::Arg(1)],
            }),
        }],
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
            vec![],
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
            // TODO conds
            vec![TransitionConstraint {
                transition_limit: 1,
                cond: None,
            }],
            vec![
                TransitionConstraint {
                    transition_limit: 5,
                    cond: None,
                },
                TransitionConstraint {
                    transition_limit: 5,
                    cond: None,
                },
            ],
            vec![TransitionConstraint {
                transition_limit: 4,
                cond: None,
            }],
            vec![
                TransitionConstraint {
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
                TransitionConstraint {
                    transition_limit: 4,
                    cond: Some(Expression {
                        args: vec![ExprArg::Storage("pp_1".into()), ExprArg::User(1)],
                        expr: EExpr::Boolean(TExpr {
                            operators: vec![Op {
                                op_type: EOp::Rel(RelOp::Eqs),
                                operands_ids: [0, 1],
                            }],
                            terms: vec![ExprTerm::Arg(0), ExprTerm::Arg(1)],
                        }),
                    }),
                },
                TransitionConstraint {
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
            vec![TransitionConstraint {
                transition_limit: 1,
                cond: None,
            }],
            vec![TransitionConstraint {
                transition_limit: 1,
                cond: None,
            }],
        ],
    }];

    let propose_setting = ProposeSettings {
        binds: vec![DataType::U128((5 * ONE_NEAR).into())],
        storage_key: "wf_bounty_1".into(),
    };

    (wfs, propose_setting)
}
