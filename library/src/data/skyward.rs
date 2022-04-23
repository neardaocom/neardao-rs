use crate::workflow::{ArgSrc, Transition};
use crate::{
    expression::{EExpr, FnName},
    unit_tests::ONE_NEAR,
    workflow::{
        ActionData, ActionType, DataType, DataTypeDef, FnCallData, FnCallMetadata, VoteScenario,
    },
    workflow::{
        ActivityRight, ExprArg, Expression, Postprocessing, PostprocessingType, ProposeSettings,
        Template, TemplateActivity, TemplateSettings,
    },
    FnCallId,
};

use super::{TemplateData, TemplateUserSettings};

pub const SKYWARD_ACC: &str = "demo-skyward.petrstudynka.testnet";
pub const WNEAR_ACC: &str = "wrap.testnet";

pub fn workflow_skyward_template_data_1() -> TemplateData {
    let pp_register_tokens = None;
    let pp_storage_deposit_1 = None;
    let pp_storage_deposit_2 = None;
    let pp_amount_send = None;

    let pp_sale_create = Some(Postprocessing {
        storage_key: "pp_5".into(),
        op_type: PostprocessingType::FnCallResult(DataTypeDef::U32(false)),
        instructions: vec![],
    });

    let wf = Template {
        code: "wf_skyward".into(),
        version: 1,
        activities: vec![
            None,
            Some(TemplateActivity {
                code: "register_tokens".into(),
                exec_condition: None,
                action: ActionType::FnCall,
                action_data: Some(ActionData::FnCall(FnCallData {
                    id: (SKYWARD_ACC.into(), "register_tokens".into()),
                    tgas: 30,
                    deposit: 20_000_000_000_000_000_000_000.into(),
                })),
                arg_types: vec![DataTypeDef::Object(0)],
                postprocessing: pp_register_tokens,
                activity_inputs: vec![vec![ArgSrc::Expression(Expression {
                    args: vec![ExprArg::Const(0), ExprArg::Bind(0)],
                    expr: EExpr::Fn(FnName::ToArray),
                })]],
                must_succeed: true,
            }),
            Some(TemplateActivity {
                code: "transfer_tokens".into(),
                exec_condition: None,
                action: ActionType::FnCall,
                action_data: Some(ActionData::FnCall(FnCallData {
                    id: ("self".into(), "storage_deposit".into()),
                    tgas: 10,
                    deposit: 1_250_000_000_000_000_000_000.into(),
                })),
                arg_types: vec![DataTypeDef::Object(0)],
                postprocessing: pp_storage_deposit_1,
                activity_inputs: vec![vec![ArgSrc::BindTpl(0)]],
                must_succeed: false,
            }),
            Some(TemplateActivity {
                code: "transfer_tokens".into(),
                exec_condition: None,
                action: ActionType::FnCall,
                action_data: Some(ActionData::FnCall(FnCallData {
                    id: (WNEAR_ACC.into(), "storage_deposit".into()), //TODO dynamic receiver
                    tgas: 10,
                    deposit: 1_250_000_000_000_000_000_000.into(),
                })),
                arg_types: vec![DataTypeDef::Object(0)],
                postprocessing: pp_storage_deposit_2,
                activity_inputs: vec![vec![ArgSrc::BindTpl(0)]],
                must_succeed: false,
            }),
            Some(TemplateActivity {
                code: "transfer_tokens".into(),
                exec_condition: None,
                action: ActionType::FnCall,
                action_data: Some(ActionData::FnCall(FnCallData {
                    id: ("self".into(), "ft_transfer_call".into()),
                    tgas: 100,
                    deposit: 1.into(),
                })),
                arg_types: vec![DataTypeDef::Object(0)],
                postprocessing: pp_amount_send,
                activity_inputs: vec![vec![
                    ArgSrc::Free,
                    ArgSrc::Bind(1),
                    ArgSrc::BindTpl(0),
                    ArgSrc::BindTpl(1),
                ]],
                must_succeed: true,
            }),
            Some(TemplateActivity {
                code: "sale_create".into(),
                exec_condition: None,
                action: ActionType::FnCall,
                action_data: Some(ActionData::FnCall(FnCallData {
                    id: (SKYWARD_ACC.into(), "sale_create".into()),
                    tgas: 50,
                    deposit: (3 * ONE_NEAR).into(),
                })),
                arg_types: vec![DataTypeDef::Object(0)],
                postprocessing: pp_sale_create,
                activity_inputs: vec![
                    vec![ArgSrc::Object(1)],
                    vec![
                        ArgSrc::Bind(2),
                        ArgSrc::Bind(3),
                        ArgSrc::Const(0),
                        ArgSrc::VecObject(2),
                        ArgSrc::Bind(0),
                        ArgSrc::Bind(4),
                        ArgSrc::Bind(5),
                    ],
                    vec![ArgSrc::Const(0), ArgSrc::Bind(1), ArgSrc::BindTpl(2)],
                ],
                must_succeed: true,
            }),
        ],
        transitions: vec![vec![1], vec![2, 3, 4], vec![3, 4], vec![4], vec![5]],
        binds: vec![
            DataType::String(SKYWARD_ACC.into()),
            DataType::String("\\\"AccountDeposit\\\"".into()),
            DataType::Null,
        ],
        start: vec![0],
        end: vec![5],
        obj_validators: vec![vec![], vec![], vec![], vec![], vec![]],
        validator_exprs: vec![],
    };

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
            vec![Transition {
                transition_limit: 1,
                cond: None,
            }],
            vec![
                Transition {
                    transition_limit: 1,
                    cond: None,
                },
                Transition {
                    transition_limit: 1,
                    cond: None,
                },
                Transition {
                    transition_limit: 1,
                    cond: None,
                },
            ],
            vec![
                Transition {
                    transition_limit: 1,
                    cond: None,
                },
                Transition {
                    transition_limit: 1,
                    cond: None,
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

    // User proposed settings type
    let settings = ProposeSettings {
        binds: vec![
            DataType::String(WNEAR_ACC.into()),
            DataType::U128(1000.into()),
            DataType::String("NearDAO auction".into()),
            DataType::String("www.neardao.com".into()),
            DataType::U64(1653304093000000000.into()),
            DataType::U64(604800000000000.into()),
        ],
        storage_key: "wf_skyward_1".into(),
    };

    (wfs, settings)
}
