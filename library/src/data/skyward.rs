use crate::{
    expression::{EExpr, EOp, ExprTerm, FnName, Op, RelOp, TExpr},
    types::{ActionIdent, DataType, DataTypeDef, FnCallMetadata, ValidatorType},
    unit_tests::ONE_NEAR,
    workflow::{
        ActivityRight, ArgType, CondOrExpr, ExprArg, Expression, Postprocessing,
        PostprocessingType, ProposeSettings, Template, TemplateActivity, TemplateSettings,
        TransitionConstraint, VoteScenario,
    },
    FnCallId,
};

pub fn workflow_skyward_template_data_1() -> (Template, Vec<FnCallId>, Vec<Vec<FnCallMetadata>>) {
    let pp_register_tokens = Some(Postprocessing {
        storage_key: "pp_1".into(),
        op_type: PostprocessingType::SaveUserValue((0, 0)),
        instructions: vec![],
    });

    let pp_storage_deposit_1 = Some(Postprocessing {
        storage_key: "pp_2".into(),
        op_type: PostprocessingType::SaveValue(DataType::Bool(true)),
        instructions: vec![],
    });

    let pp_storage_deposit_2 = Some(Postprocessing {
        storage_key: "pp_3".into(),
        op_type: PostprocessingType::SaveValue(DataType::Bool(true)),
        instructions: vec![],
    });

    let pp_amount_send = Some(Postprocessing {
        storage_key: "pp_4".into(),
        op_type: PostprocessingType::SaveUserValue((0, 0)),
        instructions: vec![],
    });

    let wf = Template {
        name: "wf_skyward".into(),
        version: 1,
        activities: vec![
            None,
            Some(TemplateActivity {
                code: "register_tokens".into(),
                exec_condition: None,
                action: ActionIdent::FnCall,
                fncall_id: Some(("skyward.near".into(), "register_tokens".into())),
                tgas: 10,
                deposit: 20_000_000_000_000_000_000_000,
                arg_types: vec![DataTypeDef::Object(0)],
                postprocessing: pp_register_tokens,
            }),
            Some(TemplateActivity {
                code: "storage_deposit".into(),
                exec_condition: None,
                action: ActionIdent::FnCall,
                fncall_id: Some(("self".into(), "storage_deposit".into())),
                tgas: 10,
                deposit: 1_250_000_000_000_000_000_000,
                arg_types: vec![DataTypeDef::Object(0)],
                postprocessing: pp_storage_deposit_1,
            }),
            Some(TemplateActivity {
                code: "storage_deposit".into(),
                exec_condition: None,
                action: ActionIdent::FnCall,
                fncall_id: Some(("wrap.near".into(), "storage_deposit".into())),
                tgas: 10,
                deposit: 1_250_000_000_000_000_000_000,
                arg_types: vec![DataTypeDef::Object(0)],
                postprocessing: pp_storage_deposit_2,
            }),
            Some(TemplateActivity {
                code: "ft_transfer_call".into(),
                exec_condition: None,
                action: ActionIdent::FnCall,
                fncall_id: Some(("self".into(), "ft_transfer_call".into())),
                tgas: 30,
                deposit: 0,
                arg_types: vec![DataTypeDef::Object(0)],
                postprocessing: pp_amount_send,
            }),
            Some(TemplateActivity {
                code: "sale_create".into(),
                exec_condition: None,
                action: ActionIdent::FnCall,
                fncall_id: Some(("skyward.near".into(), "sale_create".into())),
                tgas: 50,
                deposit: 2_000_000_000_000_000_000_000_000_000,
                arg_types: vec![DataTypeDef::Object(0)],
                postprocessing: None,
            }),
        ],
        transitions: vec![vec![1], vec![2], vec![3], vec![4], vec![5]], //TODO skip storage
        binds: vec![],
        start: vec![0],
        end: vec![5],
    };

    //TODO be able to say we use receiver as from storage
    let fncalls: Vec<FnCallId> = vec![
        ("skyward.near".into(), "register_tokens".into()),
        ("self".into(), "storage_deposit".into()),
        ("wrap.near".into(), "storage_deposit".into()),
        ("self".into(), "ft_transfer_call".into()),
        ("skyward.near".into(), "sale_create".into()),
    ];

    let metadata_1 = vec![FnCallMetadata {
        arg_names: vec!["token_account_ids".into()],
        arg_types: vec![DataTypeDef::VecString],
    }];

    let metadata_2 = vec![FnCallMetadata {
        arg_names: vec!["account_id".into()],
        arg_types: vec![DataTypeDef::String(false)],
    }];

    let metadata_3 = vec![FnCallMetadata {
        arg_names: vec!["account_id".into()],
        arg_types: vec![DataTypeDef::String(false)],
    }];

    let metadata_4 = vec![FnCallMetadata {
        arg_names: vec![
            "amount".into(),
            "memo".into(),
            "receiver_id".into(),
            "msg".into(),
        ],
        arg_types: vec![
            DataTypeDef::U128(false),
            DataTypeDef::String(true),
            DataTypeDef::String(false),
            DataTypeDef::String(false),
        ],
    }];

    let metadata_5 = vec![
        FnCallMetadata {
            arg_names: vec!["sale".into()],
            arg_types: vec![DataTypeDef::Object(1)],
        },
        FnCallMetadata {
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
                DataTypeDef::String(false),
                DataTypeDef::String(true),
                DataTypeDef::String(true),
                DataTypeDef::VecObject(2),
                DataTypeDef::String(false),
                DataTypeDef::U64(false),
                DataTypeDef::U64(false),
            ],
        },
        FnCallMetadata {
            arg_names: vec![
                "token_account_id".into(),
                "balance".into(),
                "referral_bpt".into(),
            ],
            arg_types: vec![
                DataTypeDef::String(false),
                DataTypeDef::U128(false),
                DataTypeDef::U16(true),
            ],
        },
    ];

    let metadata = vec![metadata_1, metadata_2, metadata_3, metadata_4, metadata_5];
    (wf, fncalls, metadata)
}

pub fn workflow_skyward_template_settings_data_1() -> (Vec<TemplateSettings>, ProposeSettings) {
    let wfs = vec![TemplateSettings {
        activity_rights: vec![
            vec![],
            vec![ActivityRight::GroupLeader(1)],
            vec![ActivityRight::GroupLeader(1)],
            vec![ActivityRight::GroupLeader(1)],
            vec![ActivityRight::GroupLeader(1)],
            vec![ActivityRight::GroupLeader(1)],
        ],
        allowed_proposers: vec![ActivityRight::Group(1)],
        allowed_voters: ActivityRight::TokenHolder,
        scenario: VoteScenario::TokenWeighted,
        duration: 3600,
        quorum: 51,
        approve_threshold: 20,
        spam_threshold: 80,
        vote_only_once: true,
        deposit_propose: Some(1),
        deposit_vote: Some(1000),
        deposit_propose_return: 0,
    }];

    // User proposed settings type
    let settings = ProposeSettings {
        activity_inputs: vec![
            // register tokens
            vec![vec![ArgType::Expression(Expression {
                args: vec![ExprArg::Const(0), ExprArg::User(0)],
                expr: EExpr::Fn(FnName::ArrayMerge),
            })]],
            // storage_deposit on self
            vec![vec![ArgType::Bind(0)]],
            // storage_deposit on other token
            vec![vec![ArgType::Bind(0)]],
            // ft_transfer_call on self
            vec![vec![
                ArgType::Free,
                ArgType::Free,
                ArgType::Bind(0),
                ArgType::Bind(1),
            ]],
            // sale_create
            vec![
                vec![ArgType::Object(1)],
                vec![
                    ArgType::Free,
                    ArgType::Free,
                    ArgType::Const(0),
                    ArgType::VecObject(2),
                    ArgType::Free,
                    ArgType::Free,
                    ArgType::Free,
                ],
                vec![
                    ArgType::Const(0),
                    ArgType::Storage("pp_4".into()),
                    ArgType::Free,
                ],
            ],
        ],
        transition_constraints: vec![
            vec![TransitionConstraint {
                transition_limit: 1,
                cond: None,
            }],
            vec![TransitionConstraint {
                transition_limit: 1,
                cond: None,
            }],
            vec![TransitionConstraint {
                transition_limit: 1,
                cond: None,
            }],
            vec![TransitionConstraint {
                transition_limit: 1,
                cond: None,
            }],
            vec![TransitionConstraint {
                transition_limit: 1,
                cond: None,
            }],
        ],
        binds: vec![
            DataType::String("skyward.near".into()),
            DataType::String("\\\"AccountDeposit\\\"".into()),
            DataType::U128(ONE_NEAR.into()),
        ],
        obj_validators: vec![
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            //vec![ValidatorType::Collection(2)],
        ],
        validator_exprs: vec![
        /*  We dont need to validate the value now because its binded from ft_transfer_call result
            Expression {
            args: vec![ExprArg::Bind(2), ExprArg::User(1)],
            expr: EExpr::Boolean(TExpr {
                operators: vec![Op {
                    op_type: EOp::Rel(RelOp::GtE),
                    operands_ids: [0, 1],
                }],
                terms: vec![ExprTerm::Arg(0), ExprTerm::Arg(1)],
            }),
            }
        */
        ],
        storage_key: "wf_skyward_1".into(),
    };

    (wfs, settings)
}
