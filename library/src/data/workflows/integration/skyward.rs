use std::collections::HashMap;

use near_sdk::{AccountId, ONE_NEAR, ONE_YOCTO};

use crate::{
    data::TemplateData,
    interpreter::expression::{EExpr, EOp, ExprTerm, FnName, Op, RelOp, TExpr},
    types::{
        datatype::{Datatype, Value},
        source::SourceDataVariant,
    },
    workflow::{
        action::{ActionData, FnCallData, FnCallIdType, TemplateAction},
        activity::{Activity, TemplateActivity, Terminality, Transition, TransitionLimit},
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

pub struct Skyward1TemplateOptions {
    pub skyward_account_id: String,
    pub wnear_account_id: String,
}

pub struct Skyward1ProposeOptions {
    pub token_account_id: String,
    pub token_amount: u128,
    pub auction_start: u128,
    pub auction_duration: u128,
}

pub const SKYWARD_FNCALL1_NAME: &str = "register_tokens";
pub const SKYWARD_FNCALL2_NAME: &str = "storage_deposit";
pub const SKYWARD_FNCALL3_NAME: &str = "storage_deposit";
pub const SKYWARD_FNCALL4_NAME: &str = "ft_transfer_call";
pub const SKYWARD_FNCALL5_NAME: &str = "sale_create";

pub const SKYWARD1_SETTINGS_DEPOSIT_PROPOSE: u128 = ONE_NEAR;
pub const SKYWARD1_SETTINGS_DEPOSIT_VOTE: u128 = ONE_YOCTO;

pub const SKYWARD1_STORAGE_KEY: &str = "storage_key_wf_skyward1";

pub const SKYWARD_ACC: &str = "demo-skyward.petrstudynka.testnet";
pub const WNEAR_ACC: &str = "wrap.testnet";
pub const TOKEN_ACC: &str = "my_vote_token.testnet";
pub const TOKEN_AMOUNT: u128 = 1_000_000_000;
/// Friday 1. July 2022 0:00:00 in nanos
pub const AUCTION_START: u128 = 1656633600000000000;
/// One week
pub const AUCTION_DURATION: u128 = 604800000000000;

pub const OFFERED_TOKEN_KEY: &str = "offered_token";
pub const OFFERED_TOKEN_AMOUNT_KEY: &str = "offered_amount";

/// Workflow description:
/// Workflow creates new action on skyward-finance.near.
/// Token and its amount offered on skyward and sale start is defined by proposal settings.
/// Workflow has predefined wrap.near as required token.
/// Metadata:
/// - wnear and skyward account defined by template
/// - offered token and its amount, start and duration defined by propose settings
/// - auction title and url defined by action 3 user inputs
pub struct Skyward1;
impl Skyward1 {
    pub fn template(options: Option<Skyward1TemplateOptions>) -> TemplateData {
        let (skyward_account_id, wnear_account_id) = match options {
            Some(o) => (o.skyward_account_id, o.wnear_account_id),
            None => (SKYWARD_ACC.into(), WNEAR_ACC.into()),
        };

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
        tpl_consts_map.insert(
            "account_skyward".into(),
            Value::String(skyward_account_id.clone()),
        );
        tpl_consts_map.insert(
            "account_wnear".into(),
            Value::String(wnear_account_id.clone()),
        );
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
            is_simple: false,
            need_storage: true,
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
                                AccountId::new_unchecked(skyward_account_id.clone()),
                                SKYWARD_FNCALL1_NAME.into(),
                            ),
                            tgas: 30,
                            deposit: Some(ArgSrc::ConstsTpl("deposit_register_tokens".into())),
                            binds: vec![BindDefinition {
                                key: "token_account_ids".into(),
                                key_src: SrcOrExprOrValue::Expr(Expression {
                                    args: vec![
                                        ArgSrc::ConstsTpl("account_wnear".into()),
                                        ArgSrc::ConstPropSettings(OFFERED_TOKEN_KEY.into()),
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
                                    AccountId::new_unchecked(wnear_account_id.clone()),
                                    SKYWARD_FNCALL2_NAME.into(),
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
                                    ArgSrc::ConstPropSettings(OFFERED_TOKEN_KEY.into()),
                                    SKYWARD_FNCALL3_NAME.into(),
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
                                    ArgSrc::ConstPropSettings(OFFERED_TOKEN_KEY.into()),
                                    SKYWARD_FNCALL4_NAME.into(),
                                ),
                                tgas: 100,
                                deposit: Some(ArgSrc::ConstsTpl("deposit_ft_transfer_call".into())),
                                binds: vec![
                                    BindDefinition {
                                        key: "receiver_id".into(),
                                        key_src: SrcOrExprOrValue::Src(ArgSrc::ConstsTpl(
                                            skyward_account_id.clone(),
                                        )),
                                        prefixes: vec![],
                                        is_collection: false,
                                    },
                                    BindDefinition {
                                        key: "amount".into(),
                                        key_src: SrcOrExprOrValue::Src(ArgSrc::ConstPropSettings(
                                            OFFERED_TOKEN_AMOUNT_KEY.into(),
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
                                AccountId::new_unchecked(skyward_account_id.clone()),
                                SKYWARD_FNCALL5_NAME.into(),
                            ),
                            tgas: 50,
                            deposit: Some(ArgSrc::ConstsTpl("deposit_sale_create".into())),
                            binds: vec![
                                /*                                 BindDefinition {
                                                                    key: "sale.title".into(),
                                                                    key_src: SrcOrExprOrValue::Src(ArgSrc::ConstAction(
                                                                        "title".into(),
                                                                    )),
                                                                    prefixes: vec![],
                                                                    is_collection: false,
                                                                }, */
                                /*                                 BindDefinition {
                                                                    key: "sale.url".into(),
                                                                    key_src: SrcOrExprOrValue::Src(ArgSrc::ConstAction(
                                                                        "url".into(),
                                                                    )),
                                                                    prefixes: vec![],
                                                                    is_collection: false,
                                                                }, */
                                BindDefinition {
                                    key: "sale.permissions_contract_id".into(),
                                    key_src: SrcOrExprOrValue::Src(ArgSrc::Const(0)),
                                    prefixes: vec![],
                                    is_collection: false,
                                },
                                BindDefinition {
                                    key: "token_account_id".into(),
                                    key_src: SrcOrExprOrValue::Src(ArgSrc::ConstPropSettings(
                                        OFFERED_TOKEN_KEY.into(),
                                    )),
                                    prefixes: vec!["sale.out_tokens".into()],
                                    is_collection: true,
                                },
                                BindDefinition {
                                    key: "balance".into(),
                                    key_src: SrcOrExprOrValue::Src(ArgSrc::ConstPropSettings(
                                        OFFERED_TOKEN_AMOUNT_KEY.into(),
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
                    automatic: false,
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
                AccountId::new_unchecked(skyward_account_id.clone()),
                "register_tokens".into(),
            ),
            (
                AccountId::new_unchecked(wnear_account_id.clone()),
                "sale_create".into(),
            ),
        ];

        let metadata_1 = vec![ObjectMetadata {
            arg_names: vec!["token_account_ids".into()],
            arg_types: vec![Datatype::VecString],
        }];

        let metadata_2 = vec![
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

        let metadata = vec![metadata_1, metadata_2];
        (wf, fncalls, metadata)
    }

    pub fn propose_settings(
        options: Option<Skyward1ProposeOptions>,
        storage_key: Option<&str>,
    ) -> ProposeSettings {
        let (token_account_id, token_amount, auction_start, auction_duration) = match options {
            Some(o) => (
                o.token_account_id,
                o.token_amount,
                o.auction_start,
                o.auction_duration,
            ),
            None => (
                TOKEN_ACC.into(),
                TOKEN_AMOUNT,
                AUCTION_START.into(),
                AUCTION_DURATION.into(),
            ),
        };

        let mut global_consts = HashMap::new();
        global_consts.insert(
            OFFERED_TOKEN_KEY.into(),
            Value::String(token_account_id.clone()),
        );
        global_consts.insert(
            OFFERED_TOKEN_AMOUNT_KEY.into(),
            Value::U128(token_amount.into()),
        );

        // Sale create action
        let mut sale_create_map: HashMap<String, Value> = HashMap::new();
        sale_create_map.insert(
            "start_time".into(),
            Value::U128(auction_start.into()), // TODO: U128 might not work
        );
        sale_create_map.insert("duration".into(), Value::U64(auction_duration as u64));

        // User proposed settings type
        let propose_settings = ProposeSettings {
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
            storage_key: Some(storage_key.unwrap_or(SKYWARD1_STORAGE_KEY).into()),
        };
        propose_settings
    }

    pub fn template_settings() -> TemplateSettings {
        let settings = TemplateSettings {
            activity_rights: vec![
                vec![ActivityRight::GroupLeader(1)],
                vec![ActivityRight::GroupLeader(1)],
                vec![ActivityRight::GroupLeader(1)],
                vec![ActivityRight::GroupLeader(1)],
                vec![ActivityRight::GroupLeader(1)],
            ],
            allowed_proposers: vec![ActivityRight::Group(1)],
            allowed_voters: ActivityRight::Group(1),
            scenario: VoteScenario::Democratic,
            duration: 35,
            quorum: 51,
            approve_threshold: 20,
            spam_threshold: 80,
            vote_only_once: true,
            deposit_propose: Some(Self::deposit_propose().into()),
            deposit_vote: Some(Self::deposit_vote().into()),
            deposit_propose_return: 0,
            transition_limits: vec![
                vec![TransitionLimit { to: 1, limit: 10 }],
                vec![
                    TransitionLimit { to: 2, limit: 10 },
                    TransitionLimit { to: 3, limit: 10 },
                    TransitionLimit { to: 4, limit: 10 },
                ],
                vec![
                    TransitionLimit { to: 3, limit: 10 },
                    TransitionLimit { to: 4, limit: 10 },
                ],
                vec![TransitionLimit { to: 4, limit: 10 }],
                vec![TransitionLimit { to: 5, limit: 10 }],
            ], // TODO: Update limits for production version.
            constants: None,
        };

        settings
    }

    pub fn deposit_propose() -> u128 {
        SKYWARD1_SETTINGS_DEPOSIT_PROPOSE
    }
    pub fn deposit_vote() -> u128 {
        SKYWARD1_SETTINGS_DEPOSIT_VOTE
    }
}
