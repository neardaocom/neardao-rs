use crate::{
    expression::{EExpr, FnName},
    types::{
        ActionData, ActionIdent, DataType, DataTypeDef, FnCallData, FnCallMetadata, VoteScenario,
    },
    unit_tests::ONE_NEAR,
    workflow::{
        ActivityRight, ArgType, ExprArg, Expression, Postprocessing, PostprocessingType,
        ProposeSettings, Template, TemplateActivity, TemplateSettings, TransitionConstraint,
    },
    FnCallId,
};

use super::{TemplateData, TemplateUserSettings};

pub const SKYWARD_ACC: &str = "demo-skyward.petrstudynka.testnet";
pub const WNEAR_ACC: &str = "wrap.testnet";

pub fn workflow_skyward_template_data_1() -> TemplateData {
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

    let pp_amount_send = None;

    let pp_sale_create = Some(Postprocessing {
        storage_key: "pp_5".into(),
        op_type: PostprocessingType::FnCallResult(DataTypeDef::U32(false)), //might overflow in future coz skyward returns u64 but we have U64 only
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
                action_data: Some(ActionData::FnCall(FnCallData {
                    id: (SKYWARD_ACC.into(), "register_tokens".into()),
                    tgas: 10,
                    deposit: 20_000_000_000_000_000_000_000.into(),
                })),
                arg_types: vec![DataTypeDef::Object(0)],
                postprocessing: pp_register_tokens,
                activity_inputs: vec![vec![ArgType::Expression(Expression {
                    args: vec![ExprArg::Const(0), ExprArg::User(0)],
                    expr: EExpr::Fn(FnName::ArrayMerge),
                })]],
                must_succeed: true,
            }),
            Some(TemplateActivity {
                code: "transfer_tokens".into(),
                exec_condition: None,
                action: ActionIdent::FnCall,
                action_data: Some(ActionData::FnCall(FnCallData {
                    id: ("self".into(), "storage_deposit".into()),
                    tgas: 10,
                    deposit: 1_250_000_000_000_000_000_000.into(),
                })),
                arg_types: vec![DataTypeDef::Object(0)],
                postprocessing: pp_storage_deposit_1,
                activity_inputs: vec![vec![ArgType::BindTpl(0)]],
                must_succeed: false,
            }),
            Some(TemplateActivity {
                code: "transfer_tokens".into(),
                exec_condition: None,
                action: ActionIdent::FnCall,
                action_data: Some(ActionData::FnCall(FnCallData {
                    id: (WNEAR_ACC.into(), "storage_deposit".into()),
                    tgas: 10,
                    deposit: 1_250_000_000_000_000_000_000.into(),
                })),
                arg_types: vec![DataTypeDef::Object(0)],
                postprocessing: pp_storage_deposit_2,
                activity_inputs: vec![vec![ArgType::BindTpl(0)]],
                must_succeed: false,
            }),
            Some(TemplateActivity {
                code: "transfer_tokens".into(),
                exec_condition: None,
                action: ActionIdent::FnCall,
                action_data: Some(ActionData::FnCall(FnCallData {
                    id: ("self".into(), "ft_transfer_call".into()),
                    tgas: 60,
                    deposit: 1.into(),
                })),
                arg_types: vec![DataTypeDef::Object(0)],
                postprocessing: pp_amount_send,
                activity_inputs: vec![vec![
                    ArgType::Free,
                    ArgType::Bind(0),
                    ArgType::BindTpl(0),
                    ArgType::BindTpl(1),
                ]],
                must_succeed: true,
            }),
            Some(TemplateActivity {
                code: "sale_create".into(),
                exec_condition: None,
                action: ActionIdent::FnCall,
                action_data: Some(ActionData::FnCall(FnCallData {
                    id: (SKYWARD_ACC.into(), "sale_create".into()),
                    tgas: 50,
                    deposit: (3 * ONE_NEAR).into(),
                })),
                arg_types: vec![DataTypeDef::Object(0)],
                postprocessing: pp_sale_create,
                activity_inputs: vec![
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
                    vec![ArgType::Const(0), ArgType::Bind(0), ArgType::Free],
                ],
                must_succeed: true,
            }),
        ],
        transitions: vec![vec![1], vec![2, 3, 4], vec![3, 4], vec![4], vec![5]],
        binds: vec![
            DataType::String(SKYWARD_ACC.into()),
            DataType::String("\\\"AccountDeposit\\\"".into()),
        ],
        start: vec![0],
        end: vec![5],
        obj_validators: vec![vec![], vec![], vec![], vec![], vec![]],
        validator_exprs: vec![],
    };

    //TODO be able to say we use receiver as from storage
    let fncalls: Vec<FnCallId> = vec![
        (SKYWARD_ACC.into(), "register_tokens".into()),
        ("self".into(), "storage_deposit".into()),
        (WNEAR_ACC.into(), "storage_deposit".into()),
        ("self".into(), "ft_transfer_call".into()),
        (SKYWARD_ACC.into(), "sale_create".into()),
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
            "memo".into(),
            "amount".into(),
            "receiver_id".into(),
            "msg".into(),
        ],
        arg_types: vec![
            DataTypeDef::String(true),
            DataTypeDef::U128(false),
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
        transition_constraints: vec![
            vec![TransitionConstraint {
                transition_limit: 1,
                cond: None,
            }],
            vec![
                TransitionConstraint {
                    transition_limit: 1,
                    cond: None,
                },
                TransitionConstraint {
                    transition_limit: 1,
                    cond: None,
                },
                TransitionConstraint {
                    transition_limit: 1,
                    cond: None,
                },
            ],
            vec![
                TransitionConstraint {
                    transition_limit: 1,
                    cond: None,
                },
                TransitionConstraint {
                    transition_limit: 1,
                    cond: None,
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

    // User proposed settings type
    let settings = ProposeSettings {
        binds: vec![DataType::U128(1000.into())],
        storage_key: "wf_skyward_1".into(),
    };

    (wfs, settings)
}
