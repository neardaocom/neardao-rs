use std::collections::HashMap;

use near_sdk::{AccountId, ONE_NEAR, ONE_YOCTO};

use library::{
    types::{
        datatype::{Datatype, Value},
        source::SourceDataVariant,
    },
    workflow::{
        action::{ActionType, FnCallData, FnCallIdType, TemplateAction},
        activity::{Activity, TemplateActivity, Terminality, Transition, TransitionLimit},
        postprocessing::Postprocessing,
        settings::{ActivityBind, ProposeSettings, TemplateSettings},
        template::Template,
        types::{
            ActivityRight, ArgSrc, BindDefinition, Instruction, ObjectMetadata, SrcOrExprOrValue,
            VoteScenario,
        },
    },
};

use crate::TemplateData;

pub const DEFAULT_VOTING_DURATION: u32 = 10;

pub struct WfAdd1ProposeOptions {
    pub template_id: u16,
    pub provider_id: String,
}

pub const WF_ADD1_PROVIDER_ID_KEY: &str = "provider_id";
pub const WF_ADD1_TEMPLATE_ID_KEY: &str = "workflow_id";

pub const WF_ADD1_SETTINGS_DEPOSIT_PROPOSE: u128 = ONE_NEAR;
pub const WF_ADD1_SETTINGS_DEPOSIT_VOTE: u128 = ONE_YOCTO;

/// Expects provider_id and id of template to download defined in global propose settings.
pub struct WfAdd1;
impl WfAdd1 {
    pub fn template(provider_id: String) -> TemplateData {
        let provider_id = AccountId::try_from(provider_id).expect("invalid account_id string");
        let map = HashMap::new();
        let tpl = Template {
            code: "wf_add".into(),
            version: "1".into(),
            auto_exec: true,
            need_storage: false, // TODO: Not sure if true is true.
            activities: vec![
                Activity::Init,
                Activity::Activity(TemplateActivity {
                    code: "wf_add".into(),
                    postprocessing: None,
                    actions: vec![TemplateAction {
                        exec_condition: None,
                        validators: vec![],
                        action_data: ActionType::FnCall(FnCallData {
                            id: FnCallIdType::Dynamic(
                                ArgSrc::ConstPropSettings(WF_ADD1_PROVIDER_ID_KEY.into()),
                                "wf_template".into(),
                            ),
                            tgas: 30,
                            deposit: None,
                            binds: vec![BindDefinition {
                                key: "id".into(),
                                key_src: SrcOrExprOrValue::Src(ArgSrc::ConstPropSettings(
                                    WF_ADD1_TEMPLATE_ID_KEY.into(),
                                )),
                                collection_data: None,
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
                    is_sync: false,
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
        };
        let fn_calls = vec![(provider_id, "wf_template".to_string())];
        let metadata = vec![vec![ObjectMetadata {
            arg_names: vec!["id".into()],
            arg_types: vec![Datatype::U64(false)],
        }]];
        (tpl, fn_calls, metadata)
    }
    pub fn propose_settings(options: Option<WfAdd1ProposeOptions>) -> ProposeSettings {
        let WfAdd1ProposeOptions {
            template_id,
            provider_id,
        } = options.expect("WfAddProposeOptions default options are not supported yet");
        let mut global_consts = HashMap::new();
        global_consts.insert(
            WF_ADD1_PROVIDER_ID_KEY.into(),
            Value::String(provider_id.clone()),
        );
        global_consts.insert(
            WF_ADD1_TEMPLATE_ID_KEY.into(),
            Value::U64(template_id as u64),
        );

        // User proposed settings type
        let settings = ProposeSettings {
            global: Some(SourceDataVariant::Map(global_consts)),
            binds: vec![
                None,
                Some(ActivityBind {
                    shared: None,
                    values: vec![None],
                }),
            ],
            storage_key: None,
        };
        settings
    }

    /// Default template settings for workflow: wf_add.
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
        WF_ADD1_SETTINGS_DEPOSIT_PROPOSE
    }
    pub fn deposit_vote() -> u128 {
        WF_ADD1_SETTINGS_DEPOSIT_VOTE
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
