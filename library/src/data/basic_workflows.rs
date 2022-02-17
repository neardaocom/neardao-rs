use crate::{
    expression::{EExpr, EOp, ExprTerm, FnName, Op, RelOp, TExpr},
    types::{
        ActionIdent, DataType, DataTypeDef, FnCallData, FnCallMetadata, ValidatorType, VoteScenario,
    },
    unit_tests::ONE_NEAR,
    workflow::{
        ActivityRight, ArgType, CondOrExpr, ExprArg, Expression, Postprocessing,
        PostprocessingType, ProposeSettings, Template, TemplateActivity, TemplateSettings,
        TransitionConstraint,
    },
    FnCallId,
};

pub fn workflow_wf_add() -> Template {
    Template {
        name: "wf_add".into(),
        version: 1,
        activities: vec![
            None,
            Some(TemplateActivity {
                code: "wf_add".into(),
                exec_condition: None,
                action: ActionIdent::WorkflowAdd,
                action_data: None,
                arg_types: vec![DataTypeDef::U16(false)],
                postprocessing: None,
                activity_inputs: vec![vec![ArgType::Bind(0)]],
                must_succeed: true,
            }),
        ],
        transitions: vec![vec![1]],
        start: vec![0],
        end: vec![1],
        binds: vec![],
        obj_validators: vec![vec![ValidatorType::Primitive(0)]],
        validator_exprs: vec![Expression {
            args: vec![ExprArg::User(0), ExprArg::Bind(0)],
            expr: EExpr::Boolean(TExpr {
                operators: vec![Op {
                    op_type: EOp::Rel(RelOp::Eqs),
                    operands_ids: [0, 1],
                }],
                terms: vec![ExprTerm::Arg(1), ExprTerm::Arg(0)],
            }),
        }],
    }
}

pub fn workflow_settings_wf_add() -> TemplateSettings {
    TemplateSettings {
        activity_rights: vec![vec![ActivityRight::GroupLeader(1)]],
        allowed_proposers: vec![ActivityRight::Group(1)],
        allowed_voters: ActivityRight::TokenHolder,
        scenario: VoteScenario::TokenWeighted,
        duration: 60,
        quorum: 51,
        approve_threshold: 20,
        spam_threshold: 80,
        vote_only_once: true,
        deposit_propose: Some(1.into()),
        deposit_vote: Some(1000.into()),
        deposit_propose_return: 0,
        transition_constraints: vec![vec![TransitionConstraint {
            transition_limit: 1,
            cond: None,
        }]],
    }
}

pub fn workflow_treasury_send_near_loop() -> Template {
    Template {
        name: "wf_near_send".into(),
        version: 1,
        activities: vec![
            None,
            Some(TemplateActivity {
                code: "near_send".into(),
                exec_condition: None,
                action: ActionIdent::TreasurySendNear,
                action_data: None,
                arg_types: vec![DataTypeDef::String(false), DataTypeDef::U128(false)],
                postprocessing: None,
                activity_inputs: vec![vec![ArgType::Free, ArgType::Free]],
                must_succeed: true,
            }),
        ],
        transitions: vec![vec![1], vec![1]],
        binds: vec![],
        start: vec![0],
        end: vec![1],
        obj_validators: vec![vec![ValidatorType::Primitive(0)]],
        validator_exprs: vec![Expression {
            args: vec![ExprArg::User(1), ExprArg::Bind(0)],
            expr: EExpr::Boolean(TExpr {
                operators: vec![Op {
                    op_type: EOp::Rel(RelOp::GtE),
                    operands_ids: [0, 1],
                }],
                terms: vec![ExprTerm::Arg(1), ExprTerm::Arg(0)],
            }),
        }],
    }
}

pub fn workflow_settings_treasury_send_near_loop() -> TemplateSettings {
    TemplateSettings {
        activity_rights: vec![vec![ActivityRight::GroupLeader(1)]],
        allowed_proposers: vec![ActivityRight::Group(1)],
        allowed_voters: ActivityRight::TokenHolder,
        scenario: VoteScenario::TokenWeighted,
        duration: 60,
        quorum: 51,
        approve_threshold: 20,
        spam_threshold: 80,
        vote_only_once: true,
        deposit_propose: Some(1.into()),
        deposit_vote: Some(1000.into()),
        deposit_propose_return: 0,
        transition_constraints: vec![
            vec![TransitionConstraint {
                transition_limit: 1,
                cond: None,
            }],
            vec![TransitionConstraint {
                transition_limit: 4,
                cond: None,
            }],
        ],
    }
}

pub fn workflow_treasury_send_near() -> Template {
    Template {
        name: "wf_near_send".into(),
        version: 1,
        activities: vec![
            None,
            Some(TemplateActivity {
                code: "near_send".into(),
                exec_condition: None,
                action: ActionIdent::TreasurySendNear,
                action_data: None,
                arg_types: vec![DataTypeDef::String(false), DataTypeDef::U128(false)],
                postprocessing: None,
                activity_inputs: vec![vec![ArgType::Free, ArgType::Free]],
                must_succeed: true,
            }),
        ],
        transitions: vec![vec![1]],
        binds: vec![],
        start: vec![0],
        end: vec![1],
        obj_validators: vec![vec![
            ValidatorType::Primitive(0),
            ValidatorType::Primitive(0),
        ]],
        validator_exprs: vec![
            Expression {
                args: vec![ExprArg::User(0), ExprArg::Bind(0)],
                expr: EExpr::Boolean(TExpr {
                    operators: vec![Op {
                        op_type: EOp::Rel(RelOp::Eqs),
                        operands_ids: [0, 1],
                    }],
                    terms: vec![ExprTerm::Arg(1), ExprTerm::Arg(0)],
                }),
            },
            Expression {
                args: vec![ExprArg::User(1), ExprArg::Bind(1)],
                expr: EExpr::Boolean(TExpr {
                    operators: vec![Op {
                        op_type: EOp::Rel(RelOp::GtE),
                        operands_ids: [0, 1],
                    }],
                    terms: vec![ExprTerm::Arg(1), ExprTerm::Arg(0)],
                }),
            },
        ],
    }
}

pub fn workflow_settings_treasury_send_near() -> TemplateSettings {
    TemplateSettings {
        activity_rights: vec![vec![ActivityRight::GroupLeader(1)]],
        allowed_proposers: vec![ActivityRight::Group(1)],
        allowed_voters: ActivityRight::TokenHolder,
        scenario: VoteScenario::TokenWeighted,
        duration: 60,
        quorum: 51,
        approve_threshold: 20,
        spam_threshold: 80,
        vote_only_once: true,
        deposit_propose: Some(1.into()),
        deposit_vote: Some(1000.into()),
        deposit_propose_return: 0,
        transition_constraints: vec![vec![TransitionConstraint {
            transition_limit: 1,
            cond: None,
        }]],
    }
}

pub fn workflow_treasury_send_ft() -> Template {
    Template {
        name: "wf_treasury_send_ft".into(),
        version: 1,
        activities: vec![
            None,
            Some(TemplateActivity {
                code: "treasury_send_ft".into(),
                exec_condition: None,
                action: ActionIdent::TreasurySendFt,
                action_data: None,
                arg_types: vec![
                    DataTypeDef::String(false),
                    DataTypeDef::String(false),
                    DataTypeDef::U128(false),
                    DataTypeDef::String(true),
                ],
                postprocessing: None,
                activity_inputs: vec![vec![
                    ArgType::Free,
                    ArgType::Free,
                    ArgType::Free,
                    ArgType::Free,
                ]],
                must_succeed: true,
            }),
        ],
        transitions: vec![vec![1]],
        binds: vec![],
        start: vec![0],
        end: vec![1],
        obj_validators: vec![vec![]],
        validator_exprs: vec![],
    }
}

pub fn workflow_settings_treasury_send_ft() -> TemplateSettings {
    TemplateSettings {
        activity_rights: vec![vec![ActivityRight::GroupLeader(1)]],
        allowed_proposers: vec![ActivityRight::Group(1)],
        allowed_voters: ActivityRight::TokenHolder,
        scenario: VoteScenario::TokenWeighted,
        duration: 60,
        quorum: 51,
        approve_threshold: 20,
        spam_threshold: 80,
        vote_only_once: true,
        deposit_propose: Some(1.into()),
        deposit_vote: Some(1000.into()),
        deposit_propose_return: 0,
        transition_constraints: vec![vec![TransitionConstraint {
            transition_limit: 1,
            cond: None,
        }]],
    }
}

pub fn workflow_treasury_send_ft_contract() -> Template {
    Template {
        name: "wf_treasury_send_ft_contract".into(),
        version: 1,
        activities: vec![
            None,
            Some(TemplateActivity {
                code: "treasury_send_ft_contract".into(),
                exec_condition: None,
                action: ActionIdent::TreasurySendFtContract,
                action_data: None,
                arg_types: vec![
                    DataTypeDef::String(false),
                    DataTypeDef::String(false),
                    DataTypeDef::U128(false),
                    DataTypeDef::String(true),
                    DataTypeDef::String(false),
                ],
                postprocessing: None,
                activity_inputs: vec![vec![
                    ArgType::Free,
                    ArgType::Free,
                    ArgType::Free,
                    ArgType::Free,
                    ArgType::Free,
                ]],
                must_succeed: true,
            }),
        ],
        transitions: vec![vec![1]],
        binds: vec![],
        start: vec![0],
        end: vec![1],
        obj_validators: vec![vec![]],
        validator_exprs: vec![],
    }
}

pub fn workflow_settings_treasury_send_ft_contract() -> TemplateSettings {
    TemplateSettings {
        activity_rights: vec![vec![ActivityRight::GroupLeader(1)]],
        allowed_proposers: vec![ActivityRight::Group(1)],
        allowed_voters: ActivityRight::TokenHolder,
        scenario: VoteScenario::TokenWeighted,
        duration: 60,
        quorum: 51,
        approve_threshold: 20,
        spam_threshold: 80,
        vote_only_once: true,
        deposit_propose: Some(1.into()),
        deposit_vote: Some(1000.into()),
        deposit_propose_return: 0,
        transition_constraints: vec![vec![TransitionConstraint {
            transition_limit: 1,
            cond: None,
        }]],
    }
}

pub fn workflow_group_add() -> Template {
    Template {
        name: "wf_group_add".into(),
        version: 1,
        activities: vec![
            None,
            Some(TemplateActivity {
                code: "group_add".into(),
                exec_condition: None,
                action: ActionIdent::GroupAdd,
                action_data: None,
                arg_types: vec![
                    DataTypeDef::Object(1),
                    DataTypeDef::VecObject(2),
                    DataTypeDef::Object(3),
                ],
                postprocessing: None,
                activity_inputs: vec![vec![ArgType::Free, ArgType::Free, ArgType::Free]],
                must_succeed: true,
            }),
        ],
        transitions: vec![vec![1], vec![1]],
        binds: vec![],
        start: vec![0],
        end: vec![1],
        obj_validators: vec![vec![]],
        validator_exprs: vec![],
    }
}
pub fn workflow_group_members_add() -> Template {
    Template {
        name: "wf_group_members_add".into(),
        version: 1,
        activities: vec![
            None,
            Some(TemplateActivity {
                code: "group_members_add".into(),
                exec_condition: None,
                action: ActionIdent::GroupAddMembers,
                action_data: None,
                arg_types: vec![DataTypeDef::U16(false), DataTypeDef::VecObject(1)],
                postprocessing: None,
                activity_inputs: vec![vec![ArgType::Free, ArgType::Free]],
                must_succeed: true,
            }),
        ],
        transitions: vec![vec![1]],
        binds: vec![],
        start: vec![0],
        end: vec![1],
        obj_validators: vec![vec![ValidatorType::Primitive(0)]],
        validator_exprs: vec![Expression {
            args: vec![ExprArg::User(0), ExprArg::Bind(0)],
            expr: EExpr::Boolean(TExpr {
                operators: vec![Op {
                    op_type: EOp::Rel(RelOp::Eqs),
                    operands_ids: [0, 1],
                }],
                terms: vec![ExprTerm::Arg(1), ExprTerm::Arg(0)],
            }),
        }],
    }
}

pub fn workflow_settings_group_member_add() -> TemplateSettings {
    TemplateSettings {
        activity_rights: vec![vec![ActivityRight::GroupLeader(1)]],
        allowed_proposers: vec![ActivityRight::Group(1)],
        allowed_voters: ActivityRight::TokenHolder,
        scenario: VoteScenario::TokenWeighted,
        duration: 60,
        quorum: 51,
        approve_threshold: 20,
        spam_threshold: 80,
        vote_only_once: true,
        deposit_propose: Some(1.into()),
        deposit_vote: Some(1000.into()),
        deposit_propose_return: 0,
        transition_constraints: vec![vec![TransitionConstraint {
            transition_limit: 1,
            cond: None,
        }]],
    }
}

pub fn workflow_group_remove() -> Template {
    Template {
        name: "wf_group_remove".into(),
        version: 1,
        activities: vec![
            None,
            Some(TemplateActivity {
                code: "group_remove".into(),
                exec_condition: None,
                action: ActionIdent::GroupRemove,
                action_data: None,
                arg_types: vec![DataTypeDef::U16(false)],
                postprocessing: None,
                activity_inputs: vec![vec![ArgType::Free]],
                must_succeed: true,
            }),
        ],
        transitions: vec![vec![1], vec![1]],
        binds: vec![],
        start: vec![0],
        end: vec![1],
        obj_validators: vec![vec![]],
        validator_exprs: vec![],
    }
}
pub fn workflow_group_member_remove() -> Template {
    Template {
        name: "wf_group_member_remove".into(),
        version: 1,
        activities: vec![
            None,
            Some(TemplateActivity {
                code: "group_member_remove".into(),
                exec_condition: None,
                action: ActionIdent::GroupRemoveMember,
                action_data: None,
                arg_types: vec![DataTypeDef::U16(false), DataTypeDef::String(false)],
                postprocessing: None,
                activity_inputs: vec![vec![ArgType::Free, ArgType::Free]],
                must_succeed: true,
            }),
        ],
        transitions: vec![vec![1], vec![1]],
        binds: vec![],
        start: vec![0],
        end: vec![1],
        obj_validators: vec![vec![]],
        validator_exprs: vec![],
    }
}
pub fn workflow_tag_add() -> Template {
    Template {
        name: "wf_tag_add".into(),
        version: 1,
        activities: vec![
            None,
            Some(TemplateActivity {
                code: "tag_add".into(),
                exec_condition: None,
                action: ActionIdent::TagAdd,
                action_data: None,
                arg_types: vec![DataTypeDef::String(false), DataTypeDef::VecString],
                postprocessing: None,
                activity_inputs: vec![vec![ArgType::Free, ArgType::Free]],
                must_succeed: true,
            }),
        ],
        transitions: vec![vec![1], vec![1]],
        binds: vec![],
        start: vec![0],
        end: vec![1],
        obj_validators: vec![vec![]],
        validator_exprs: vec![],
    }
}
pub fn workflow_tag_edit() -> Template {
    Template {
        name: "wf_tag_edit".into(),
        version: 1,
        activities: vec![
            None,
            Some(TemplateActivity {
                code: "wf_tag_edit".into(),
                exec_condition: None,
                action: ActionIdent::TagRemove,
                action_data: None,
                arg_types: vec![
                    DataTypeDef::String(false),
                    DataTypeDef::U16(false),
                    DataTypeDef::String(false),
                ],
                postprocessing: None,
                activity_inputs: vec![vec![ArgType::Free, ArgType::Free, ArgType::Free]],
                must_succeed: true,
            }),
        ],
        transitions: vec![vec![1], vec![1]],
        binds: vec![],
        start: vec![0],
        end: vec![1],
        obj_validators: vec![vec![]],
        validator_exprs: vec![],
    }
}
pub fn workflow_ft_distribute() -> Template {
    Template {
        name: "wf_ft_distribute".into(),
        version: 1,
        activities: vec![
            None,
            Some(TemplateActivity {
                code: "ft_distribute".into(),
                exec_condition: None,
                action: ActionIdent::FtDistribute,
                action_data: None,
                arg_types: vec![
                    DataTypeDef::U16(false),
                    DataTypeDef::U32(false),
                    DataTypeDef::VecString,
                ],
                postprocessing: None,
                activity_inputs: vec![vec![ArgType::Free, ArgType::Free, ArgType::Free]],
                must_succeed: true,
            }),
        ],
        transitions: vec![vec![1]],
        binds: vec![],
        start: vec![0],
        end: vec![1],
        obj_validators: vec![vec![]],
        validator_exprs: vec![],
    }
}

pub fn workflow_media_add() -> Template {
    Template {
        name: "wf_media_add".into(),
        version: 1,
        activities: vec![
            None,
            Some(TemplateActivity {
                code: "media_add".into(),
                exec_condition: None,
                action: ActionIdent::MediaAdd,
                action_data: None,
                arg_types: vec![DataTypeDef::Object(1)],
                postprocessing: None,
                activity_inputs: vec![vec![ArgType::Free]],
                must_succeed: true,
            }),
        ],
        transitions: vec![vec![1]],
        binds: vec![],
        start: vec![0],
        end: vec![1],
        obj_validators: vec![vec![]],
        validator_exprs: vec![],
    }
}
