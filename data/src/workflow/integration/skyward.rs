use std::collections::HashMap;

use near_sdk::{AccountId, ONE_NEAR, ONE_YOCTO};

use crate::{
    object_metadata::standard_fn_calls::{NEP_141_FT_TRANSFER_CALL, NEP_145_STORAGE_DEPOSIT},
    TemplateData,
};

use library::{
    interpreter::expression::{EExpr, EOp, ExprTerm, FnName, Op, RelOp, TExpr},
    types::{
        datatype::{Datatype, Value},
        source::SourceDataVariant,
    },
    workflow::{
        action::{ActionData, FnCallData, FnCallIdType, InputSource, TemplateAction},
        activity::{Activity, TemplateActivity, Terminality, Transition, TransitionLimit},
        expression::Expression,
        postprocessing::Postprocessing,
        settings::{ActivityBind, ProposeSettings, TemplateSettings},
        template::Template,
        types::{
            ActivityRight, BindDefinition, CollectionBindData, CollectionBindingStyle,
            FnCallResultType, Instruction, ObjectMetadata, Src, ValueSrc, VoteScenario,
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
pub const SKYWARD_FNCALL2_NAME: &str = NEP_145_STORAGE_DEPOSIT;
pub const SKYWARD_FNCALL3_NAME: &str = NEP_145_STORAGE_DEPOSIT;
pub const SKYWARD_FNCALL4_NAME: &str = NEP_141_FT_TRANSFER_CALL;
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

/// Check with mainnet version of the contract.
pub const WNEAR_STORAGE_DEPOSIT_AMOUNT: u128 = 1_250_000_000_000_000_000_000;

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
                "pp_3_result".into(),
                Value::Bool(true),
            )],
        });
        let pp_sale_create = Some(Postprocessing {
            instructions: vec![Instruction::StoreFnCallResult(
                "skyward_auction_id".into(),
                FnCallResultType::Datatype(Datatype::U64(false)),
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
            code: "skyward1".into(),
            version: "1".into(),
            auto_exec: false,
            need_storage: true,
            receiver_storage_keys: vec![],
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
                            deposit: Some(ValueSrc::Src(Src::Tpl(
                                "deposit_register_tokens".into(),
                            ))),
                            binds: vec![BindDefinition {
                                key: "token_account_ids".into(),
                                value: ValueSrc::Expr(Expression {
                                    args: vec![
                                        Src::Tpl("account_wnear".into()),
                                        Src::PropSettings(OFFERED_TOKEN_KEY.into()),
                                    ],
                                    expr_id: 0,
                                }),
                                collection_data: None,
                            }],
                            must_succeed: true,
                        }),
                        postprocessing: pp_register_tokens,
                        optional: false,
                        input_source: InputSource::User,
                    }],
                    automatic: true,
                    terminal: Terminality::NonTerminal,
                    is_sync: false,
                }),
                Activity::Activity(TemplateActivity {
                    code: "storage_deposit".into(),
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
                                deposit: Some(ValueSrc::Src(Src::Tpl("deposit_storage".into()))),
                                binds: vec![BindDefinition {
                                    key: "account_id".into(),
                                    value: ValueSrc::Src(Src::Tpl("account_skyward".into())),
                                    collection_data: None,
                                }],
                                must_succeed: false,
                            }),
                            postprocessing: pp_storage_deposit_1,
                            optional: true,
                            input_source: InputSource::User,
                        },
                        TemplateAction {
                            exec_condition: None,
                            validators: vec![],
                            action_data: ActionData::FnCall(FnCallData {
                                id: FnCallIdType::StandardDynamic(
                                    ValueSrc::Src(Src::PropSettings(OFFERED_TOKEN_KEY.into())),
                                    SKYWARD_FNCALL3_NAME.into(),
                                ),
                                tgas: 10,
                                deposit: Some(ValueSrc::Src(Src::Tpl("deposit_storage".into()))),
                                binds: vec![BindDefinition {
                                    key: "account_id".into(),
                                    value: ValueSrc::Src(Src::Tpl("account_skyward".into())),
                                    collection_data: None,
                                }],
                                must_succeed: false,
                            }),
                            postprocessing: pp_storage_deposit_2,

                            optional: true,
                            input_source: InputSource::User,
                        },
                    ],
                    automatic: true,
                    terminal: Terminality::NonTerminal,
                    is_sync: false,
                }),
                // Taken from previous because of gas limit.
                Activity::Activity(TemplateActivity {
                    code: "transfer_tokens".into(),
                    postprocessing: None,
                    actions: vec![TemplateAction {
                        exec_condition: None,
                        validators: vec![],
                        action_data: ActionData::FnCall(FnCallData {
                            id: FnCallIdType::StandardDynamic(
                                ValueSrc::Src(Src::PropSettings(OFFERED_TOKEN_KEY.into())),
                                SKYWARD_FNCALL4_NAME.into(),
                            ),
                            tgas: 100,
                            deposit: Some(ValueSrc::Src(Src::Tpl(
                                "deposit_ft_transfer_call".into(),
                            ))),
                            binds: vec![
                                BindDefinition {
                                    key: "receiver_id".into(),
                                    value: ValueSrc::Src(Src::Tpl("account_skyward".into())),
                                    collection_data: None,
                                },
                                BindDefinition {
                                    key: "amount".into(),
                                    value: ValueSrc::Src(Src::PropSettings(
                                        OFFERED_TOKEN_AMOUNT_KEY.into(),
                                    )),
                                    collection_data: None,
                                },
                                BindDefinition {
                                    key: "msg".into(),
                                    value: ValueSrc::Src(Src::Tpl("ft_transfer_call_msg".into())),
                                    collection_data: None,
                                },
                                BindDefinition {
                                    key: "memo".into(),
                                    value: ValueSrc::Value(Value::Null),
                                    collection_data: None,
                                },
                            ],
                            must_succeed: true,
                        }),
                        postprocessing: pp_ft_transfer_call,
                        optional: false,
                        input_source: InputSource::User,
                    }],
                    automatic: true,
                    terminal: Terminality::NonTerminal,
                    is_sync: false,
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
                            deposit: Some(ValueSrc::Src(Src::Tpl("deposit_sale_create".into()))),
                            binds: vec![
                                BindDefinition {
                                    key: "sale.permissions_contract_id".into(),
                                    value: ValueSrc::Src(Src::Runtime(0)),
                                    collection_data: None,
                                },
                                BindDefinition {
                                    key: "token_account_id".into(),
                                    value: ValueSrc::Src(Src::PropSettings(
                                        OFFERED_TOKEN_KEY.into(),
                                    )),
                                    collection_data: Some(CollectionBindData {
                                        prefixes: vec!["sale.out_tokens".into()],
                                        collection_binding_type: CollectionBindingStyle::ForceSame(
                                            1,
                                        ),
                                    }),
                                },
                                BindDefinition {
                                    key: "balance".into(),
                                    value: ValueSrc::Src(Src::PropSettings(
                                        OFFERED_TOKEN_AMOUNT_KEY.into(),
                                    )),
                                    collection_data: Some(CollectionBindData {
                                        prefixes: vec!["sale.out_tokens".into()],
                                        collection_binding_type: CollectionBindingStyle::ForceSame(
                                            1,
                                        ),
                                    }),
                                },
                                BindDefinition {
                                    key: "referral_bpt".into(),
                                    value: ValueSrc::Value(Value::Null),
                                    collection_data: Some(CollectionBindData {
                                        prefixes: vec!["sale.out_tokens".into()],
                                        collection_binding_type: CollectionBindingStyle::ForceSame(
                                            1,
                                        ),
                                    }),
                                },
                                BindDefinition {
                                    key: "sale.in_token_account_id".into(),
                                    value: ValueSrc::Src(Src::Tpl("account_wnear".into())),
                                    collection_data: None,
                                },
                                BindDefinition {
                                    key: "sale.start_time".into(),
                                    value: ValueSrc::Src(Src::Action("start_time".into())),
                                    collection_data: None,
                                },
                                BindDefinition {
                                    key: "sale.duration".into(),
                                    value: ValueSrc::Src(Src::Action("duration".into())),
                                    collection_data: None,
                                },
                            ],
                            must_succeed: true,
                        }),
                        postprocessing: pp_sale_create,
                        optional: false,
                        input_source: InputSource::User,
                    }],
                    automatic: false,
                    terminal: Terminality::Automatic,
                    is_sync: false,
                }),
            ],
            expressions: vec![
                EExpr::Fn(FnName::ArrayMerge),
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
                        cond: Some(ValueSrc::Expr(Expression {
                            args: vec![Src::Storage("pp_1_result".into())],
                            expr_id: 1,
                        })),
                        time_from_cond: None,
                        time_to_cond: None,
                    },
                    Transition {
                        activity_id: 3,
                        cond: Some(ValueSrc::Expr(Expression {
                            args: vec![Src::Storage("pp_1_result".into())],
                            expr_id: 1,
                        })),
                        time_from_cond: None,
                        time_to_cond: None,
                    },
                ],
                vec![Transition {
                    activity_id: 3,
                    cond: None,
                    time_from_cond: None,
                    time_to_cond: None,
                }],
                vec![Transition {
                    activity_id: 4,
                    cond: Some(ValueSrc::Expr(Expression {
                        args: vec![Src::Storage("pp_3_result".into())],
                        expr_id: 1,
                    })),
                    time_from_cond: None,
                    time_to_cond: None,
                }],
            ],
            constants: SourceDataVariant::Map(tpl_consts_map),
            end: vec![4],
        };

        let fncalls: Vec<FnCallId> = vec![
            (
                AccountId::new_unchecked(skyward_account_id.clone()),
                "register_tokens".into(),
            ),
            (
                AccountId::new_unchecked(skyward_account_id.clone()),
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
                    Datatype::U128(false),
                    Datatype::U128(false),
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
        let std_fncalls = vec![
            NEP_145_STORAGE_DEPOSIT.into(),
            NEP_141_FT_TRANSFER_CALL.into(),
        ];
        (wf, fncalls, metadata, std_fncalls)
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
        sale_create_map.insert("start_time".into(), Value::U128(auction_start.into()));
        sale_create_map.insert("duration".into(), Value::U64(auction_duration as u64));

        // User proposed settings type
        let propose_settings = ProposeSettings {
            constants: Some(SourceDataVariant::Map(global_consts)),
            activity_constants: vec![
                None,
                None,
                None,
                None,
                Some(ActivityBind {
                    constants: None,
                    actions_constants: vec![Some(SourceDataVariant::Map(sale_create_map))],
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
                ],
                vec![TransitionLimit { to: 3, limit: 10 }],
                vec![TransitionLimit { to: 4, limit: 10 }],
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

/// TODO: Finish
/// Same as `Skyward1` except it has merged 2. activity with second 3.
pub struct Skyward2;
impl Skyward2 {
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
                "pp_3_result".into(),
                Value::Bool(true),
            )],
        });
        let pp_sale_create = Some(Postprocessing {
            instructions: vec![Instruction::StoreFnCallResult(
                "skyward_auction_id".into(),
                FnCallResultType::Datatype(Datatype::U64(false)),
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
            auto_exec: false,
            need_storage: true,
            receiver_storage_keys: vec![],
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
                            deposit: Some(ValueSrc::Src(Src::Tpl(
                                "deposit_register_tokens".into(),
                            ))),
                            binds: vec![BindDefinition {
                                key: "token_account_ids".into(),
                                value: ValueSrc::Expr(Expression {
                                    args: vec![
                                        Src::Tpl("account_wnear".into()),
                                        Src::PropSettings(OFFERED_TOKEN_KEY.into()),
                                    ],
                                    expr_id: 0,
                                }),
                                collection_data: None,
                            }],
                            must_succeed: true,
                        }),
                        postprocessing: pp_register_tokens,
                        optional: false,
                        input_source: InputSource::User,
                    }],
                    automatic: true,
                    terminal: Terminality::NonTerminal,
                    is_sync: false,
                }),
                Activity::Activity(TemplateActivity {
                    code: "storage_deposit".into(),
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
                                deposit: Some(ValueSrc::Src(Src::Tpl("deposit_storage".into()))),
                                binds: vec![BindDefinition {
                                    key: "account_id".into(),
                                    value: ValueSrc::Src(Src::Tpl("account_skyward".into())),
                                    collection_data: None,
                                }],
                                must_succeed: false,
                            }),
                            postprocessing: pp_storage_deposit_1,
                            optional: true,
                            input_source: InputSource::User,
                        },
                        TemplateAction {
                            exec_condition: None,
                            validators: vec![],
                            action_data: ActionData::FnCall(FnCallData {
                                id: FnCallIdType::StandardDynamic(
                                    ValueSrc::Src(Src::PropSettings(OFFERED_TOKEN_KEY.into())),
                                    SKYWARD_FNCALL3_NAME.into(),
                                ),
                                tgas: 10,
                                deposit: Some(ValueSrc::Src(Src::Tpl("deposit_storage".into()))),
                                binds: vec![BindDefinition {
                                    key: "account_id".into(),
                                    value: ValueSrc::Src(Src::Tpl("account_skyward".into())),
                                    collection_data: None,
                                }],
                                must_succeed: false,
                            }),
                            postprocessing: pp_storage_deposit_2,
                            optional: true,
                            input_source: InputSource::User,
                        },
                    ],
                    automatic: true,
                    terminal: Terminality::NonTerminal,
                    is_sync: false,
                }),
                // Taken from previous because of gas limit.
                Activity::Activity(TemplateActivity {
                    code: "transfer_tokens".into(),
                    postprocessing: None,
                    actions: vec![TemplateAction {
                        exec_condition: None,
                        validators: vec![],
                        action_data: ActionData::FnCall(FnCallData {
                            id: FnCallIdType::StandardDynamic(
                                ValueSrc::Src(Src::PropSettings(OFFERED_TOKEN_KEY.into())),
                                SKYWARD_FNCALL4_NAME.into(),
                            ),
                            tgas: 100,
                            deposit: Some(ValueSrc::Src(Src::Tpl(
                                "deposit_ft_transfer_call".into(),
                            ))),
                            binds: vec![
                                BindDefinition {
                                    key: "receiver_id".into(),
                                    value: ValueSrc::Src(Src::Tpl("account_skyward".into())),
                                    collection_data: None,
                                },
                                BindDefinition {
                                    key: "amount".into(),
                                    value: ValueSrc::Src(Src::PropSettings(
                                        OFFERED_TOKEN_AMOUNT_KEY.into(),
                                    )),
                                    collection_data: None,
                                },
                                BindDefinition {
                                    key: "msg".into(),
                                    value: ValueSrc::Src(Src::Tpl("ft_transfer_call_msg".into())),
                                    collection_data: None,
                                },
                                BindDefinition {
                                    key: "memo".into(),
                                    value: ValueSrc::Value(Value::Null),
                                    collection_data: None,
                                },
                            ],
                            must_succeed: true,
                        }),
                        postprocessing: pp_ft_transfer_call,
                        optional: false,
                        input_source: InputSource::User,
                    }],
                    automatic: true,
                    terminal: Terminality::NonTerminal,
                    is_sync: false,
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
                            deposit: Some(ValueSrc::Src(Src::Tpl("deposit_sale_create".into()))),
                            binds: vec![
                                BindDefinition {
                                    key: "sale.permissions_contract_id".into(),
                                    value: ValueSrc::Src(Src::Runtime(0)),
                                    collection_data: None,
                                },
                                BindDefinition {
                                    key: "token_account_id".into(),
                                    value: ValueSrc::Src(Src::PropSettings(
                                        OFFERED_TOKEN_KEY.into(),
                                    )),
                                    collection_data: Some(CollectionBindData {
                                        prefixes: vec!["sale.out_tokens".into()],
                                        collection_binding_type: CollectionBindingStyle::ForceSame(
                                            1,
                                        ),
                                    }),
                                },
                                BindDefinition {
                                    key: "balance".into(),
                                    value: ValueSrc::Src(Src::PropSettings(
                                        OFFERED_TOKEN_AMOUNT_KEY.into(),
                                    )),
                                    collection_data: Some(CollectionBindData {
                                        prefixes: vec!["sale.out_tokens".into()],
                                        collection_binding_type: CollectionBindingStyle::ForceSame(
                                            1,
                                        ),
                                    }),
                                },
                                BindDefinition {
                                    key: "referral_bpt".into(),
                                    value: ValueSrc::Value(Value::Null),
                                    collection_data: Some(CollectionBindData {
                                        prefixes: vec!["sale.out_tokens".into()],
                                        collection_binding_type: CollectionBindingStyle::ForceSame(
                                            1,
                                        ),
                                    }),
                                },
                                BindDefinition {
                                    key: "sale.in_token_account_id".into(),
                                    value: ValueSrc::Src(Src::Tpl("account_wnear".into())),
                                    collection_data: None,
                                },
                                BindDefinition {
                                    key: "sale.start_time".into(),
                                    value: ValueSrc::Src(Src::Action("start_time".into())),
                                    collection_data: None,
                                },
                                BindDefinition {
                                    key: "sale.duration".into(),
                                    value: ValueSrc::Src(Src::Action("duration".into())),
                                    collection_data: None,
                                },
                            ],
                            must_succeed: true,
                        }),
                        postprocessing: pp_sale_create,
                        optional: false,
                        input_source: InputSource::User,
                    }],
                    automatic: false,
                    terminal: Terminality::User,
                    is_sync: false,
                }),
            ],
            expressions: vec![
                EExpr::Fn(FnName::ArrayMerge),
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
                        cond: Some(ValueSrc::Expr(Expression {
                            args: vec![Src::Storage("pp_1_result".into())],
                            expr_id: 1,
                        })),
                        time_from_cond: None,
                        time_to_cond: None,
                    },
                    Transition {
                        activity_id: 3,
                        cond: Some(ValueSrc::Expr(Expression {
                            args: vec![Src::Storage("pp_1_result".into())],
                            expr_id: 1,
                        })),
                        time_from_cond: None,
                        time_to_cond: None,
                    },
                ],
                vec![Transition {
                    activity_id: 3,
                    cond: None,
                    time_from_cond: None,
                    time_to_cond: None,
                }],
                vec![Transition {
                    activity_id: 4,
                    cond: Some(ValueSrc::Expr(Expression {
                        args: vec![Src::Storage("pp_3_result".into())],
                        expr_id: 1,
                    })),
                    time_from_cond: None,
                    time_to_cond: None,
                }],
            ],
            constants: SourceDataVariant::Map(tpl_consts_map),
            end: vec![4],
        };

        let fncalls: Vec<FnCallId> = vec![
            (
                AccountId::new_unchecked(skyward_account_id.clone()),
                "register_tokens".into(),
            ),
            (
                AccountId::new_unchecked(skyward_account_id.clone()),
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
                    Datatype::U128(false),
                    Datatype::U128(false),
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
        let std_fncalls = vec![
            NEP_145_STORAGE_DEPOSIT.into(),
            NEP_141_FT_TRANSFER_CALL.into(),
        ];
        (wf, fncalls, metadata, std_fncalls)
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
        sale_create_map.insert("start_time".into(), Value::U128(auction_start.into()));
        sale_create_map.insert("duration".into(), Value::U64(auction_duration as u64));

        // User proposed settings type
        let propose_settings = ProposeSettings {
            constants: Some(SourceDataVariant::Map(global_consts)),
            activity_constants: vec![
                None,
                None,
                None,
                None,
                Some(ActivityBind {
                    constants: None,
                    actions_constants: vec![Some(SourceDataVariant::Map(sale_create_map))],
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
                ],
                vec![TransitionLimit { to: 3, limit: 10 }],
                vec![TransitionLimit { to: 4, limit: 10 }],
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
