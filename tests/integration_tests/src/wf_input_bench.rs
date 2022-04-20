use anyhow::Result;
use library::{
    interpreter::expression::{EExpr, EOp, ExprTerm, Op, RelOp, TExpr},
    types::{
        activity_input::InputHashMap,
        datatype::{Datatype, Value},
    },
    workflow::{
        expression::Expression,
        types::{ArgSrc, FnCallMetadata, ValidatorRef, ValidatorType},
    },
    ObjectValues,
};
use serde_json::json;
use simple_dao::{TestCaseNew, TestCaseOld};
use workspaces::network::DevAccountDeployer;

use crate::utils::outcome_pretty;

#[tokio::test]
async fn bench_wf_input() -> Result<()> {
    let worker = workspaces::sandbox().await?;
    let wasm_blob = workspaces::compile_project("./../mocks/simple_dao").await?;
    let contract = worker.dev_deploy(&wasm_blob).await?;

    let testcases_old_version: Vec<(String, TestCaseOld)> = vec![
        ("small".into(), old_small_sized_testcase()),
        ("big".into(), old_big_sized_testcase()),
    ];

    let testcases_new_version: Vec<(String, TestCaseNew)> = vec![
        //("small".into(), new_small_sized_testcase()),
        //("big".into(), new_big_sized_testcase()),
    ];

    let args = json!({
        "testcases_old" : testcases_old_version,
        "testcases_new" : testcases_new_version,
    })
    .to_string()
    .into_bytes();

    let outcome = contract
        .call(&worker, "new")
        .args(args)
        .max_gas()
        .transact()
        .await?;

    assert!(outcome.is_success());

    // Testcases

    let args = json!({
        "testcase" : "small",
        "input": old_small_user_input()
    })
    .to_string()
    .into_bytes();
    let outcome = contract
        .call(&worker, "bench_wf_old")
        .args(args)
        .max_gas()
        .transact()
        .await?;
    assert!(outcome.is_success());
    outcome_pretty("old_small", outcome);

    let args = json!({
        "testcase" : "big",
        "input": old_big_user_input()
    })
    .to_string()
    .into_bytes();
    let outcome = contract
        .call(&worker, "bench_wf_old")
        .args(args)
        .max_gas()
        .transact()
        .await?;
    assert!(outcome.is_success());
    outcome_pretty("old_big", outcome);

    // NEW version

    /*     let args = json!({
        "testcase" : "small",
        "input": new_small_user_input()
    })
    .to_string()
    .into_bytes();
    let outcome = contract
        .call(&worker, "bench_wf_new")
        .args(args)
        .max_gas()
        .transact()
        .await?;
    assert!(outcome.is_success());
    outcome_pretty("new_small", outcome);

    let args = json!({
        "testcase" : "big",
        "input": new_small_user_input()
    })
    .to_string()
    .into_bytes();
    let outcome = contract
        .call(&worker, "bench_wf_new")
        .args(args)
        .max_gas()
        .transact()
        .await?;
    assert!(outcome.is_success());
    outcome_pretty("new_big", outcome); */

    Ok(())
}

fn old_small_sized_testcase() -> TestCaseOld {
    let fncall_metadata = vec![FnCallMetadata {
        arg_names: vec![
            "title".into(),
            "url".into(),
            "permissions_contract_id".into(),
            "in_token_account_id".into(),
            "start_time".into(),
            "duration".into(),
        ],
        arg_types: vec![
            Datatype::String(false),
            Datatype::String(true),
            Datatype::String(true),
            Datatype::String(false),
            Datatype::U128(false),
            Datatype::U128(false),
        ],
    }];

    let validator_refs: Vec<ValidatorRef> = vec![
        ValidatorRef {
            v_type: ValidatorType::Simple,
            obj_id: 0,
            val_id: 0,
        },
        ValidatorRef {
            v_type: ValidatorType::Simple,
            obj_id: 0,
            val_id: 1,
        },
    ];

    let validators: Vec<Expression> = vec![
        Expression {
            args: vec![ArgSrc::ConstsTpl(3), ArgSrc::User(4)],
            expr: EExpr::Boolean(TExpr {
                operators: vec![Op {
                    op_type: EOp::Rel(RelOp::Eqs),
                    operands_ids: [0, 1],
                }],
                terms: vec![ExprTerm::Arg(0), ExprTerm::Arg(1)],
            }),
        },
        Expression {
            args: vec![ArgSrc::ConstsTpl(4), ArgSrc::User(5)],
            expr: EExpr::Boolean(TExpr {
                operators: vec![Op {
                    op_type: EOp::Rel(RelOp::Eqs),
                    operands_ids: [0, 1],
                }],
                terms: vec![ExprTerm::Arg(0), ExprTerm::Arg(1)],
            }),
        },
    ];

    let expressions: Vec<Expression> = vec![];

    let source_defs = vec![vec![
        ArgSrc::ConstsTpl(0),
        ArgSrc::ConstsTpl(1),
        ArgSrc::ConstsTpl(2),
        ArgSrc::User(3),
        ArgSrc::User(4),
        ArgSrc::User(5),
    ]];

    let tpl_consts: Vec<Value> = vec![
        Value::String("from_const_0".into()),
        Value::String("from_const_1".into()),
        Value::String("from_const_2".into()),
        Value::U128(0.into()),
        Value::U128(3600.into()),
    ];

    TestCaseOld {
        fncall_metadata,
        validator_refs,
        validators,
        expressions,
        source_defs,
        tpl_consts,
    }
}

fn old_small_user_input() -> ObjectValues {
    let user_input = vec![vec![
        Value::String("user_input_0".into()),
        Value::String("user_input_1".into()),
        Value::String("user_input_2".into()),
        Value::String("user_input_3".into()),
        Value::U128(0.into()),
        Value::U128(3600.into()),
    ]];

    user_input
}

fn old_big_user_input() -> ObjectValues {
    let user_input = vec![
        vec![Value::Null, Value::String("test".into())],
        vec![
            Value::String("Neardao token auction".into()),
            Value::String("www.neardao.com".into()),
            Value::String("neardao.testnet".into()),
            Value::Null,
            Value::String("wrap.near".into()),
            Value::U128(0.into()),
            Value::U128(3600.into()),
            Value::Null,
        ],
        vec![Value::String("testing".into()), Value::U64(420)],
        vec![
            Value::String("neardao.testnet".into()),
            Value::U128(999.into()), // 1000 is limit
            Value::Null,
            Value::Null,
            Value::String("neardao.testnet".into()),
            Value::U128(991.into()), // 1000 is limit
            Value::Null,
            Value::Null,
            Value::String("neardao.testnet".into()),
            Value::U128(991.into()), // 1000 is limit
            Value::Null,
            Value::Null,
        ],
        vec![
            Value::String("petr.near".into()),
            Value::U64(123), // 500 is limit
            Value::String("david.near".into()),
            Value::U64(456), // 500 is limit
            Value::String("tomas.near".into()),
            Value::U64(456), // 500 is limit
        ],
    ];

    user_input
}

fn old_big_sized_testcase() -> TestCaseOld {
    let fncall_metadata = vec![
        FnCallMetadata {
            arg_names: vec!["sale".into(), "sale_info".into()],
            arg_types: vec![Datatype::Object(1), Datatype::String(true)],
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
                "meta".into(),
            ],
            arg_types: vec![
                Datatype::String(false),
                Datatype::String(true),
                Datatype::String(true),
                Datatype::VecObject(3),
                Datatype::String(false),
                Datatype::U128(false),
                Datatype::U128(false),
                Datatype::Object(2),
            ],
        },
        FnCallMetadata {
            arg_names: vec!["reason".into(), "timestamp".into()],
            arg_types: vec![Datatype::String(false), Datatype::U64(false)],
        },
        FnCallMetadata {
            arg_names: vec![
                "token_account_id".into(),
                "balance".into(),
                "referral_bpt".into(),
                "shares".into(),
            ],
            arg_types: vec![
                Datatype::String(false),
                Datatype::U128(false),
                Datatype::U64(true),
                Datatype::VecObject(4),
            ],
        },
        FnCallMetadata {
            arg_names: vec!["user".into(), "amount".into()],
            arg_types: vec![Datatype::String(false), Datatype::U64(false)],
        },
    ];
    let validator_refs: Vec<ValidatorRef> = vec![
        ValidatorRef {
            v_type: ValidatorType::Simple,
            obj_id: 1,
            val_id: 0,
        },
        ValidatorRef {
            v_type: ValidatorType::Collection,
            obj_id: 3,
            val_id: 1,
        },
        ValidatorRef {
            v_type: ValidatorType::Collection,
            obj_id: 4,
            val_id: 2,
        },
    ];

    let validators: Vec<Expression> = vec![
        Expression {
            args: vec![ArgSrc::ConstsTpl(0), ArgSrc::User(2)],
            expr: EExpr::Boolean(TExpr {
                operators: vec![Op {
                    op_type: EOp::Rel(RelOp::Eqs),
                    operands_ids: [0, 1],
                }],
                terms: vec![ExprTerm::Arg(0), ExprTerm::Arg(1)],
            }),
        },
        Expression {
            args: vec![ArgSrc::ConstsTpl(5), ArgSrc::User(1)],
            expr: EExpr::Boolean(TExpr {
                operators: vec![Op {
                    op_type: EOp::Rel(RelOp::Gt),
                    operands_ids: [0, 1],
                }],
                terms: vec![ExprTerm::Arg(0), ExprTerm::Arg(1)],
            }),
        },
        Expression {
            args: vec![ArgSrc::ConstsTpl(2), ArgSrc::User(1)],
            expr: EExpr::Boolean(TExpr {
                operators: vec![Op {
                    op_type: EOp::Rel(RelOp::Gt),
                    operands_ids: [0, 1],
                }],
                terms: vec![ExprTerm::Arg(0), ExprTerm::Arg(1)],
            }),
        },
    ];

    let expressions: Vec<Expression> = vec![];

    let source_defs = vec![
        vec![ArgSrc::Object(1), ArgSrc::ConstsTpl(3)],
        vec![
            ArgSrc::User(0),
            ArgSrc::User(1),
            ArgSrc::User(2),
            ArgSrc::VecObject(3),
            ArgSrc::User(4),
            ArgSrc::User(5),
            ArgSrc::User(6),
            ArgSrc::Object(2),
        ],
        vec![ArgSrc::ConstsTpl(4), ArgSrc::User(1)],
        vec![
            ArgSrc::ConstsTpl(1),
            ArgSrc::User(1),
            ArgSrc::User(2),
            ArgSrc::VecObject(4),
        ],
        vec![ArgSrc::User(0), ArgSrc::User(1)],
    ];

    let tpl_consts: Vec<Value> = vec![
        Value::String("neardao.testnet".into()),
        Value::String("neardao.near".into()),
        Value::U64(500),
        Value::String("info binded".into()),
        Value::String("testing binded".into()),
        Value::U128(1000.into()),
    ];

    TestCaseOld {
        fncall_metadata,
        validator_refs,
        validators,
        expressions,
        source_defs,
        tpl_consts,
    }
}

fn new_small_user_input() -> InputHashMap {
    todo!()
}

fn new_big_user_input() -> InputHashMap {
    todo!()
}

fn new_small_sized_testcase() -> TestCaseNew {
    todo!()
}

fn new_big_sized_testcase() -> TestCaseNew {
    todo!()
}
