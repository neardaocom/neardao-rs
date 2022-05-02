use std::{collections::HashMap, hash::Hash};

use near_sdk::{AccountId, ONE_NEAR};

use crate::{
    interpreter::expression::{EExpr, EOp, ExprTerm, FnName, Op, RelOp, TExpr},
    types::{
        datatype::{Datatype, Value},
        source::SourceDataVariant,
    },
    workflow::{
        action::{ActionData, FnCallData, FnCallIdType, TemplateAction},
        activity::{Activity, TemplateActivity, Terminality, Transition},
        expression::Expression,
        postprocessing::Postprocessing,
        settings::{ActivityBind, ProposeSettings, TemplateSettings},
        template::Template,
        types::{
            ActivityRight, ArgSrc, BindDefinition, Instruction, ObjectMetadata, SrcOrExprOrValue,
            VoteScenario,
        },
    },
    FnCallId,
};

use super::{TemplateData, TemplateUserSettings};

pub const SKYWARD_ACC: &str = "demo-skyward.petrstudynka.testnet";
pub const WNEAR_ACC: &str = "wrap.testnet";
pub const TOKEN_ACC: &str = "my_vote_token.testnet";
pub const TOKEN_AMOUNT: u128 = 1_000_000_000;
/// Friday 1. July 2022 0:00:00 in nanos
pub const AUCTION_START: u128 = 1656633600000000000;
/// One week
pub const AUCTION_DURATION: u128 = 604800000000000;

/// Workflow description:
/// Workflow creates new action on skyward-finance.near.
/// Token and its amount offered on skyward and sale start is defined by proposal settings.
/// Workflow has predefined wrap.near as required token.
pub fn workflow_skyward_template_data_1() -> TemplateData {
    let pp_register_tokens = Some(Postprocessing {
        instructions: vec![Instruction::StoreValue(
            "pp_1_result".into(),
            Value::Bool(true),
        )],
    });
    let pp_storage_deposit_1 = None;
    let pp_storage_deposit_2 = None;
    let pp_ft_transfer_call = Some(Postprocessing {
        instructions: vec![Instruction::StoreValue(
            "pp_4_result".into(),
            Value::Bool(true),
        )],
    });
    let pp_sale_create = Some(Postprocessing {
        instructions: vec![Instruction::StoreFnCallResult(
            "skyward_auction_id".into(),
            Datatype::U64(false),
        )],
    });

    let mut tpl_consts_map = HashMap::new();
    tpl_consts_map.insert("account_skyward".into(), Value::String(SKYWARD_ACC.into()));
    tpl_consts_map.insert("account_wnear".into(), Value::String(WNEAR_ACC.into()));
    tpl_consts_map.insert(
        "ft_transfer_call_msg".into(),
        Value::String("\\\"AccountDeposit\\\"".into()),
    );
    tpl_consts_map.insert(
        "deposit_register_tokens".into(),
        Value::U128(20_000_000_000_000_000_000_000.into()),
    );
    tpl_consts_map.insert(
        "deposit_storage".into(),
        Value::U128(1_250_000_000_000_000_000_000.into()),
    );
    tpl_consts_map.insert(
        "deposit_sale_create".into(),
        Value::U128((3 * ONE_NEAR).into()),
    );
    tpl_consts_map.insert("deposit_ft_transfer_call".into(), Value::U128(1.into()));

    let wf = Template {
        code: "wf_skyward".into(),
        version: "1".into(),
        is_simple: todo!(),
        need_storage: todo!(),
        activities: vec![
            Activity::Init,
            Activity::Activity(TemplateActivity {
                code: "register_tokens".into(),
                postprocessing: None,
                actions: vec![TemplateAction {
                    exec_condition: None,
                    validators: vec![],
                    action_data: ActionData::FnCall(FnCallData {
                        id: FnCallIdType::Static(
                            AccountId::new_unchecked(SKYWARD_ACC.into()),
                            "register_tokens".into(),
                        ),
                        tgas: 30,
                        deposit: Some(ArgSrc::ConstsTpl("deposit_register_tokens".into())),
                        binds: vec![BindDefinition {
                            key: "token_account_ids".into(),
                            key_src: SrcOrExprOrValue::Expr(Expression {
                                args: vec![
                                    ArgSrc::ConstsTpl("account_wnear".into()),
                                    ArgSrc::ConstPropSettings("offered_token".into()),
                                ],
                                expr_id: 0,
                            }),
                            prefixes: vec![],
                            is_collection: false,
                        }],
                    }),
                    postprocessing: pp_register_tokens,
                    must_succeed: true,
                    optional: false,
                }],
                automatic: true,
                terminal: Terminality::NonTerminal,
                is_dao_activity: false,
            }),
            Activity::Activity(TemplateActivity {
                code: "transfer_tokens".into(),
                postprocessing: None,
                actions: vec![
                    TemplateAction {
                        exec_condition: None,
                        validators: vec![],
                        action_data: ActionData::FnCall(FnCallData {
                            id: FnCallIdType::StandardStatic(
                                AccountId::new_unchecked(WNEAR_ACC.into()),
                                "storage_deposit".into(),
                            ),
                            tgas: 10,
                            deposit: Some(ArgSrc::ConstsTpl("deposit_storage".into())),
                            binds: vec![BindDefinition {
                                key: "account_id".into(),
                                key_src: SrcOrExprOrValue::Src(ArgSrc::ConstsTpl(
                                    "account_skyward".into(),
                                )),
                                prefixes: vec![],
                                is_collection: false,
                            }],
                        }),
                        postprocessing: pp_storage_deposit_1,
                        must_succeed: false,
                        optional: true,
                    },
                    TemplateAction {
                        exec_condition: None,
                        validators: vec![],
                        action_data: ActionData::FnCall(FnCallData {
                            id: FnCallIdType::StandardDynamic(
                                ArgSrc::ConstAction(TOKEN_ACC.into()),
                                "storage_deposit".into(),
                            ),
                            tgas: 10,
                            deposit: Some(ArgSrc::ConstsTpl("deposit_storage".into())),
                            binds: vec![BindDefinition {
                                key: "account_id".into(),
                                key_src: SrcOrExprOrValue::Src(ArgSrc::ConstsTpl(
                                    "account_skyward".into(),
                                )),
                                prefixes: vec![],
                                is_collection: false,
                            }],
                        }),
                        postprocessing: pp_storage_deposit_2,
                        must_succeed: false,
                        optional: true,
                    },
                    TemplateAction {
                        exec_condition: None,
                        validators: vec![],
                        action_data: ActionData::FnCall(FnCallData {
                            id: FnCallIdType::StandardDynamic(
                                ArgSrc::ConstAction(TOKEN_ACC.into()),
                                "ft_transfer_call".into(),
                            ),
                            tgas: 100,
                            deposit: Some(ArgSrc::ConstsTpl("deposit_ft_transfer_call".into())),
                            binds: vec![
                                BindDefinition {
                                    key: "receiver_id".into(),
                                    key_src: SrcOrExprOrValue::Src(ArgSrc::ConstsTpl(
                                        "offered_token".into(),
                                    )),
                                    prefixes: vec![],
                                    is_collection: false,
                                },
                                BindDefinition {
                                    key: "amount".into(),
                                    key_src: SrcOrExprOrValue::Src(ArgSrc::ConstAction(
                                        "offered_token".into(),
                                    )),
                                    prefixes: vec![],
                                    is_collection: false,
                                },
                                BindDefinition {
                                    key: "msg".into(),
                                    key_src: SrcOrExprOrValue::Src(ArgSrc::ConstsTpl(
                                        "ft_transfer_call_msg".into(),
                                    )),
                                    prefixes: vec![],
                                    is_collection: false,
                                },
                                BindDefinition {
                                    key: "memo".into(),
                                    key_src: SrcOrExprOrValue::Value(Value::Null),
                                    prefixes: vec![],
                                    is_collection: false,
                                },
                            ],
                        }),
                        postprocessing: pp_ft_transfer_call,
                        must_succeed: true,
                        optional: false,
                    },
                ],
                automatic: true,
                terminal: Terminality::NonTerminal,
                is_dao_activity: false,
            }),
            Activity::Activity(TemplateActivity {
                code: "sale_create".into(),
                postprocessing: None,
                actions: vec![TemplateAction {
                    exec_condition: None,
                    validators: vec![],
                    action_data: ActionData::FnCall(FnCallData {
                        id: FnCallIdType::Static(
                            AccountId::new_unchecked(SKYWARD_ACC.into()),
                            "sale_create".into(),
                        ),
                        tgas: 50,
                        deposit: Some(ArgSrc::ConstsTpl("deposit_sale_create".into())),
                        binds: vec![
                            BindDefinition {
                                key: "sale.title".into(),
                                key_src: SrcOrExprOrValue::Src(ArgSrc::ConstAction("title".into())),
                                prefixes: vec![],
                                is_collection: false,
                            },
                            BindDefinition {
                                key: "sale.url".into(),
                                key_src: SrcOrExprOrValue::Src(ArgSrc::ConstAction("url".into())),
                                prefixes: vec![],
                                is_collection: false,
                            },
                            BindDefinition {
                                key: "sale.permissions_contract_id".into(),
                                key_src: SrcOrExprOrValue::Src(ArgSrc::Const(0)),
                                prefixes: vec![],
                                is_collection: false,
                            },
                            BindDefinition {
                                key: "token_account_id".into(),
                                key_src: SrcOrExprOrValue::Src(ArgSrc::ConstPropSettings(
                                    "offered_token".into(),
                                )),
                                prefixes: vec!["sale.out_tokens".into()],
                                is_collection: true,
                            },
                            BindDefinition {
                                key: "token_account_id".into(),
                                key_src: SrcOrExprOrValue::Src(ArgSrc::ConstPropSettings(
                                    "offered_token".into(),
                                )),
                                prefixes: vec!["sale.out_tokens".into()],
                                is_collection: true,
                            },
                            BindDefinition {
                                key: "balance".into(),
                                key_src: SrcOrExprOrValue::Src(ArgSrc::ConstPropSettings(
                                    "offered_amount".into(),
                                )),
                                prefixes: vec!["sale.out_tokens".into()],
                                is_collection: true,
                            },
                            BindDefinition {
                                key: "referral_bpt".into(),
                                key_src: SrcOrExprOrValue::Value(Value::Null),
                                prefixes: vec!["sale.out_tokens".into()],
                                is_collection: true,
                            },
                            BindDefinition {
                                key: "sale.in_token_account_id".into(),
                                key_src: SrcOrExprOrValue::Src(ArgSrc::ConstsTpl(
                                    "account_wnear".into(),
                                )),
                                prefixes: vec![],
                                is_collection: false,
                            },
                            BindDefinition {
                                key: "sale.start_time".into(),
                                key_src: SrcOrExprOrValue::Src(ArgSrc::ConstAction(
                                    "start_time".into(),
                                )),
                                prefixes: vec![],
                                is_collection: false,
                            },
                            BindDefinition {
                                key: "sale.duration".into(),
                                key_src: SrcOrExprOrValue::Src(ArgSrc::ConstAction(
                                    "duration".into(),
                                )),
                                prefixes: vec![],
                                is_collection: false,
                            },
                        ],
                    }),
                    postprocessing: pp_sale_create,
                    must_succeed: true,
                    optional: false,
                }],
                automatic: true,
                terminal: Terminality::User,
                is_dao_activity: false,
            }),
        ],
        expressions: vec![
            EExpr::Fn(FnName::ToArray),
            EExpr::Boolean(TExpr {
                operators: vec![Op {
                    operands_ids: [0, 1],
                    op_type: EOp::Rel(RelOp::Eqs),
                }],
                terms: vec![ExprTerm::Arg(0), ExprTerm::Value(Value::Bool(true))],
            }),
        ],
        transitions: vec![
            vec![Transition {
                activity_id: 1,
                cond: None,
                time_from_cond: None,
                time_to_cond: None,
            }],
            vec![
                Transition {
                    activity_id: 2,
                    cond: Some(Expression {
                        args: vec![ArgSrc::Storage("pp_1_result".into())],
                        expr_id: 1,
                    }),
                    time_from_cond: None,
                    time_to_cond: None,
                },
                Transition {
                    activity_id: 3,
                    cond: Some(Expression {
                        args: vec![ArgSrc::Storage("pp_1_result".into())],
                        expr_id: 1,
                    }),
                    time_from_cond: None,
                    time_to_cond: None,
                },
                Transition {
                    activity_id: 4,
                    cond: Some(Expression {
                        args: vec![ArgSrc::Storage("pp_1_result".into())],
                        expr_id: 1,
                    }),
                    time_from_cond: None,
                    time_to_cond: None,
                },
            ],
            vec![
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
            ],
            vec![Transition {
                activity_id: 4,
                cond: None,
                time_from_cond: None,
                time_to_cond: None,
            }],
            vec![Transition {
                activity_id: 5,
                cond: Some(Expression {
                    args: vec![ArgSrc::Storage("pp_4_result".into())],
                    expr_id: 1,
                }),
                time_from_cond: None,
                time_to_cond: None,
            }],
        ],
        constants: SourceDataVariant::Map(tpl_consts_map),
        end: vec![5],
    };

    let fncalls: Vec<FnCallId> = vec![
        (
            AccountId::new_unchecked(SKYWARD_ACC.into()),
            "register_tokens".into(),
        ),
        (
            AccountId::new_unchecked(SKYWARD_ACC.into()),
            "sale_create".into(),
        ),
    ];

    let metadata_1 = vec![ObjectMetadata {
        arg_names: vec!["token_account_ids".into()],
        arg_types: vec![Datatype::VecString],
    }];

    let metadata_2 = vec![ObjectMetadata {
        arg_names: vec![
            "memo".into(),
            "amount".into(),
            "receiver_id".into(),
            "msg".into(),
        ],
        arg_types: vec![
            Datatype::String(true),
            Datatype::U128(false),
            Datatype::String(false),
            Datatype::String(false),
        ],
    }];

    let metadata_3 = vec![
        ObjectMetadata {
            arg_names: vec!["sale".into()],
            arg_types: vec![Datatype::Object(1)],
        },
        ObjectMetadata {
            arg_names: vec![
                "title".into(),
                "url".into(),
                "permissions_contract_id".into(),
                "out_tokens".into(),
                "in_token_account_id".into(),
                "start_time".into(),
                "duration".into(),
            ],
            arg_types: vec![
                Datatype::String(false),
                Datatype::String(true),
                Datatype::String(true),
                Datatype::VecObject(2),
                Datatype::String(false),
                Datatype::U64(false),
                Datatype::U64(false),
            ],
        },
        ObjectMetadata {
            arg_names: vec![
                "token_account_id".into(),
                "balance".into(),
                "referral_bpt".into(),
            ],
            arg_types: vec![
                Datatype::String(false),
                Datatype::U128(false),
                Datatype::U64(true),
            ],
        },
    ];

    let metadata = vec![metadata_1, metadata_2, metadata_3];
    (wf, fncalls, metadata)
}

pub fn workflow_skyward_template_settings_data_1() -> TemplateUserSettings {
    let wfs = vec![TemplateSettings {
        activity_rights: vec![
            vec![ActivityRight::GroupLeader(1)],
            vec![ActivityRight::GroupLeader(1)],
            vec![ActivityRight::GroupLeader(1)],
            vec![ActivityRight::GroupLeader(1)],
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
        transition_limits: vec![vec![10], vec![10, 10, 10], vec![10, 10], vec![10], vec![10]], // TODO: Update limits for production version.
        constants: None,
    }];

    let mut global_consts = HashMap::new();
    global_consts.insert("offered_token".into(), Value::String(TOKEN_ACC.into()));
    global_consts.insert("offered_amount".into(), Value::U128(TOKEN_AMOUNT.into()));

    // Sale create action
    let mut sale_create_map = HashMap::new();
    sale_create_map.insert("title".into(), Value::String("NearDAO auction".into()));
    sale_create_map.insert("url".into(), Value::String("www.neardao.com".into()));
    sale_create_map.insert(
        "start_time".into(),
        Value::U128(AUCTION_START.into()), // TODO: U128 might not work
    );
    sale_create_map.insert("duration".into(), Value::U64(AUCTION_DURATION as u64));

    // User proposed settings type
    let settings = ProposeSettings {
        global: Some(SourceDataVariant::Map(global_consts)),
        binds: vec![
            None,
            None,
            None,
            None,
            Some(ActivityBind {
                shared: None,
                values: vec![Some(SourceDataVariant::Map(sale_create_map))],
            }),
        ],
        storage_key: Some("wf_skyward_1".into()),
    };

    (wfs, settings)
}
