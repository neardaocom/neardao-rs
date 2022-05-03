pub fn workflow_wf_add() -> Template {
    let map = HashMap::new();
    Template {
        code: "wf_add".into(),
        version: "1".into(),
        is_simple: true,
        need_storage: true,
        activities: vec![
            Activity::Init,
            Activity::Activity(TemplateActivity {
                code: "wf_add".into(),
                postprocessing: None,
                actions: vec![TemplateAction {
                    exec_condition: None,
                    validators: vec![],
                    action_data: ActionData::FnCall(FnCallData {
                        id: FnCallIdType::Dynamic(
                            ArgSrc::ConstPropSettings("provider_id".into()),
                            "wf_template".into(),
                        ),
                        tgas: 30,
                        deposit: None,
                        binds: vec![BindDefinition {
                            key: "id".into(),
                            key_src: SrcOrExprOrValue::Src(ArgSrc::ConstPropSettings(
                                "workflow_id".into(),
                            )),
                            prefixes: vec![],
                            is_collection: false,
                        }],
                    }),
                    postprocessing: Some(Postprocessing {
                        instructions: vec![Instruction::StoreWorkflow],
                    }),
                    must_succeed: true,
                    optional: false,
                }],
                automatic: true,
                terminal: Terminality::Automatic,
                is_dao_activity: false,
            }),
        ],
        expressions: vec![],
        transitions: vec![vec![Transition {
            activity_id: 1,
            cond: None,
            time_from_cond: None,
            time_to_cond: None,
        }]],
        constants: SourceDataVariant::Map(map),
        end: vec![1],
    }
}

/// Default testing template settings for workflow: wf_add.
pub fn workflow_settings_wf_add() -> TemplateSettings {
    TemplateSettings {
        allowed_proposers: vec![ActivityRight::Group(1)],
        allowed_voters: ActivityRight::Group(1),
        activity_rights: vec![vec![ActivityRight::Group(1)]],
        transition_limits: vec![vec![1]],
        scenario: VoteScenario::Democratic,
        duration: 60,
        quorum: 51,
        approve_threshold: 20,
        spam_threshold: 80,
        vote_only_once: true,
        deposit_propose: Some(ONE_NEAR.into()),
        deposit_vote: Some(ONE_YOCTO.into()),
        deposit_propose_return: 0,
        constants: None,
    }
}
/// Default testing template settings for workflow: skyward.
pub fn workflow_settings_skyward_test() -> TemplateSettings {
    TemplateSettings {
        allowed_proposers: vec![ActivityRight::Group(1)],
        allowed_voters: ActivityRight::Group(1),
        activity_rights: vec![vec![ActivityRight::Group(1)]],
        transition_limits: vec![vec![1]],
        scenario: VoteScenario::Democratic,
        duration: 60,
        quorum: 51,
        approve_threshold: 20,
        spam_threshold: 80,
        vote_only_once: true,
        deposit_propose: Some(ONE_NEAR.into()),
        deposit_vote: Some(ONE_YOCTO.into()),
        deposit_propose_return: 0,
        constants: None,
    }
}

// TODO: Move to new version

/*

pub fn workflow_treasury_send_near_loop() -> Template {
    Template {
        code: "wf_near_send".into(),
        version: 1,
        activities: vec![
            None,
            Some(TemplateActivity {
                code: "near_send".into(),
                exec_condition: None,
                action: ActionType::TreasurySendNear,
                action_data: None,
                arg_types: vec![DataTypeDef::String(false), DataTypeDef::U128(false)],
                postprocessing: None,
                activity_inputs: vec![vec![ArgSrc::Free, ArgSrc::Free]],
                must_succeed: true,
            }),
        ],
        transitions: vec![vec![1], vec![1]],
        binds: vec![],
        start: vec![0],
        end: vec![1],
        obj_validators: vec![vec![ValidatorType::Simple(0)]],
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
            vec![Transition {
                transition_limit: 1,
                cond: None,
            }],
            vec![Transition {
                transition_limit: 4,
                cond: None,
            }],
        ],
    }
}

pub fn workflow_treasury_send_near() -> Template {
    Template {
        code: "wf_near_send".into(),
        version: 1,
        activities: vec![
            None,
            Some(TemplateActivity {
                code: "near_send".into(),
                exec_condition: None,
                action: ActionType::TreasurySendNear,
                action_data: None,
                arg_types: vec![DataTypeDef::String(false), DataTypeDef::U128(false)],
                postprocessing: None,
                activity_inputs: vec![vec![ArgSrc::Free, ArgSrc::Free]],
                must_succeed: true,
            }),
        ],
        transitions: vec![vec![1]],
        binds: vec![],
        start: vec![0],
        end: vec![1],
        obj_validators: vec![vec![ValidatorType::Simple(0), ValidatorType::Simple(0)]],
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

pub fn workflow_settings_basic() -> TemplateSettings {
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
        transition_constraints: vec![vec![Transition {
            transition_limit: 1,
            cond: None,
        }]],
    }
}

pub fn workflow_treasury_send_ft() -> Template {
    Template {
        code: "wf_treasury_send_ft".into(),
        version: 1,
        activities: vec![
            None,
            Some(TemplateActivity {
                code: "treasury_send_ft".into(),
                exec_condition: None,
                action: ActionType::TreasurySendFt,
                action_data: None,
                arg_types: vec![
                    DataTypeDef::String(false),
                    DataTypeDef::String(false),
                    DataTypeDef::U128(false),
                    DataTypeDef::String(true),
                ],
                postprocessing: None,
                activity_inputs: vec![vec![
                    ArgSrc::Free,
                    ArgSrc::Free,
                    ArgSrc::Free,
                    ArgSrc::Free,
                ]],
                must_succeed: true,
            }),
        ],
        transitions: vec![vec![1]],
        binds: vec![],
        start: vec![0],
        end: vec![1],
        obj_validators: vec![vec![
            ValidatorType::Simple(0),
            ValidatorType::Simple(0),
            ValidatorType::Simple(0),
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
                        op_type: EOp::Rel(RelOp::Eqs),
                        operands_ids: [0, 1],
                    }],
                    terms: vec![ExprTerm::Arg(1), ExprTerm::Arg(0)],
                }),
            },
            Expression {
                args: vec![ExprArg::User(2), ExprArg::Bind(2)],
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
        transition_constraints: vec![vec![Transition {
            transition_limit: 1,
            cond: None,
        }]],
    }
}

pub fn workflow_treasury_send_ft_contract() -> Template {
    Template {
        code: "wf_treasury_send_ft_contract".into(),
        version: 1,
        activities: vec![
            None,
            Some(TemplateActivity {
                code: "treasury_send_ft_contract".into(),
                exec_condition: None,
                action: ActionType::TreasurySendFtContract,
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
                    ArgSrc::Free,
                    ArgSrc::Free,
                    ArgSrc::Free,
                    ArgSrc::Free,
                    ArgSrc::Free,
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
        transition_constraints: vec![vec![Transition {
            transition_limit: 1,
            cond: None,
        }]],
    }
}

pub fn workflow_group_add() -> Template {
    Template {
        code: "wf_group_add".into(),
        version: 1,
        activities: vec![
            None,
            Some(TemplateActivity {
                code: "group_add".into(),
                exec_condition: None,
                action: ActionType::GroupAdd,
                action_data: None,
                arg_types: vec![
                    DataTypeDef::Object(1),
                    DataTypeDef::VecObject(2),
                    DataTypeDef::Object(3),
                ],
                postprocessing: None,
                activity_inputs: vec![vec![ArgSrc::Free, ArgSrc::Free, ArgSrc::Free]],
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
        code: "wf_group_members_add".into(),
        version: 1,
        activities: vec![
            None,
            Some(TemplateActivity {
                code: "group_members_add".into(),
                exec_condition: None,
                action: ActionType::GroupAddMembers,
                action_data: None,
                arg_types: vec![DataTypeDef::U16(false), DataTypeDef::VecObject(1)],
                postprocessing: None,
                activity_inputs: vec![vec![ArgSrc::Free, ArgSrc::Free]],
                must_succeed: true,
            }),
        ],
        transitions: vec![vec![1]],
        binds: vec![],
        start: vec![0],
        end: vec![1],
        obj_validators: vec![vec![ValidatorType::Simple(0)]],
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
        transition_constraints: vec![vec![Transition {
            transition_limit: 1,
            cond: None,
        }]],
    }
}

pub fn workflow_group_remove() -> Template {
    Template {
        code: "wf_group_remove".into(),
        version: 1,
        activities: vec![
            None,
            Some(TemplateActivity {
                code: "group_remove".into(),
                exec_condition: None,
                action: ActionType::GroupRemove,
                action_data: None,
                arg_types: vec![DataTypeDef::U16(false)],
                postprocessing: None,
                activity_inputs: vec![vec![ArgSrc::Free]],
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
        code: "wf_group_member_remove".into(),
        version: 1,
        activities: vec![
            None,
            Some(TemplateActivity {
                code: "group_member_remove".into(),
                exec_condition: None,
                action: ActionType::GroupRemoveMember,
                action_data: None,
                arg_types: vec![DataTypeDef::U16(false), DataTypeDef::String(false)],
                postprocessing: None,
                activity_inputs: vec![vec![ArgSrc::Free, ArgSrc::Free]],
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
        code: "wf_tag_add".into(),
        version: 1,
        activities: vec![
            None,
            Some(TemplateActivity {
                code: "tag_add".into(),
                exec_condition: None,
                action: ActionType::TagAdd,
                action_data: None,
                arg_types: vec![DataTypeDef::String(false), DataTypeDef::VecString],
                postprocessing: None,
                activity_inputs: vec![vec![ArgSrc::Free, ArgSrc::Free]],
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
        code: "wf_tag_edit".into(),
        version: 1,
        activities: vec![
            None,
            Some(TemplateActivity {
                code: "wf_tag_edit".into(),
                exec_condition: None,
                action: ActionType::TagRemove,
                action_data: None,
                arg_types: vec![
                    DataTypeDef::String(false),
                    DataTypeDef::U16(false),
                    DataTypeDef::String(false),
                ],
                postprocessing: None,
                activity_inputs: vec![vec![ArgSrc::Free, ArgSrc::Free, ArgSrc::Free]],
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
        code: "wf_ft_distribute".into(),
        version: 1,
        activities: vec![
            None,
            Some(TemplateActivity {
                code: "ft_distribute".into(),
                exec_condition: None,
                action: ActionType::FtDistribute,
                action_data: None,
                arg_types: vec![
                    DataTypeDef::U16(false),
                    DataTypeDef::U32(false),
                    DataTypeDef::VecString,
                ],
                postprocessing: None,
                activity_inputs: vec![vec![ArgSrc::Free, ArgSrc::Free, ArgSrc::Free]],
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
        code: "wf_media_add".into(),
        version: 1,
        activities: vec![
            None,
            Some(TemplateActivity {
                code: "media_add".into(),
                exec_condition: None,
                action: ActionType::MediaAdd,
                action_data: None,
                arg_types: vec![DataTypeDef::Object(1)],
                postprocessing: None,
                activity_inputs: vec![vec![ArgSrc::Free]],
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

pub fn workflow_media_invalidate() -> Template {
    Template {
        code: "wf_media_invalidate".into(),
        version: 1,
        activities: vec![
            None,
            Some(TemplateActivity {
                code: "media_invalidate".into(),
                exec_condition: None,
                action: ActionType::MediaInvalidate,
                action_data: None,
                arg_types: vec![DataTypeDef::U16(false)],
                postprocessing: None,
                activity_inputs: vec![vec![ArgSrc::Free]],
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
 */

use std::collections::HashMap;

use near_sdk::{AccountId, ONE_NEAR, ONE_YOCTO};

use crate::{
    types::source::SourceDataVariant,
    workflow::{
        action::{ActionData, FnCallData, FnCallIdType, TemplateAction},
        activity::{Activity, TemplateActivity, Terminality, Transition},
        expression::Expression,
        postprocessing::Postprocessing,
        settings::{ProposeSettings, TemplateSettings},
        template::Template,
        types::{
            ActivityRight, ArgSrc, BindDefinition, Instruction, SrcOrExprOrValue, VoteScenario,
        },
    },
};

use super::skyward::{workflow_skyward_template_settings_data_1, SKYWARD_ACC};
