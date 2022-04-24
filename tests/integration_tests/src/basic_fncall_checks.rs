use anyhow::Result;
use library::{
    interpreter::expression::{EExpr, EOp, ExprTerm, FnName, Op, RelOp, TExpr},
    types::{
        activity_input::{ActivityInput, InputHashMap, UserInput},
        datatype::{Datatype, Value},
    },
    workflow::{
        expression::Expression,
        types::{ArgSrc, BindDefinition, ObjectMetadata, SrcOrExpr},
        validator::{CollectionValidator, ObjectValidator, Validator},
    },
};
use serde_json::json;
use simple_dao::TestCase;
use workspaces::network::DevAccountDeployer;

use crate::utils::outcome_pretty;

#[tokio::test]
async fn skyward() -> Result<()> {
    let worker = workspaces::testnet().await?;
    let wasm_blob = workspaces::compile_project("./../mocks/simple_dao").await?;
    let contract = worker.dev_deploy(&wasm_blob).await?;

    let testcases: Vec<(String, TestCase)> = vec![("skyward".into(), testcase_skyward())];

    let args = json!({
        "testcases" : testcases,
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
    outcome_pretty("new", outcome);

    let args = json!({
        "testcase" : "skyward",
        "input": input_skyward()
    })
    .to_string()
    .into_bytes();
    let outcome = contract
        .call(&worker, "validate_bind_serialize")
        .args(args)
        .max_gas()
        .transact()
        .await?;
    assert!(outcome.is_success());
    outcome_pretty("skyward", outcome);

    Ok(())
}

fn testcase_skyward() -> TestCase {
    let fncall_metadata = vec![
        ObjectMetadata {
            arg_names: vec!["sale".into(), "sale_info".into()],
            arg_types: vec![Datatype::Object(1), Datatype::String(true)],
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
        ObjectMetadata {
            arg_names: vec!["reason".into(), "timestamp".into()],
            arg_types: vec![Datatype::String(false), Datatype::U64(false)],
        },
        ObjectMetadata {
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
                Datatype::Object(4),
            ],
        },
        ObjectMetadata {
            arg_names: vec!["user".into(), "amount".into()],
            arg_types: vec![Datatype::String(false), Datatype::U64(false)],
        },
    ];

    let tpl_consts = vec![
        (
            "sale.permissions_contract_id".into(),
            Value::String("neardao.testnet".into()),
        ),
        (
            "sale.out_tokens.token_account_id".into(),
            Value::String("neardao.near".into()),
        ),
        ("sale.out_tokens.balance".into(), Value::U128(1000.into())),
        ("sale.out_tokens.shares.amount".into(), Value::U64(500)),
        ("sale_info".into(), Value::String("sale info binded".into())),
        (
            "sale.meta.reason".into(),
            Value::String("meta reason binded".into()),
        ),
    ];
    let expressions = vec![
        EExpr::Boolean(TExpr {
            operators: vec![Op {
                op_type: EOp::Rel(RelOp::Eqs),
                operands_ids: [0, 1],
            }],
            terms: vec![ExprTerm::Arg(0), ExprTerm::Arg(1)],
        }),
        EExpr::Boolean(TExpr {
            operators: vec![Op {
                op_type: EOp::Rel(RelOp::Gt),
                operands_ids: [0, 1],
            }],
            terms: vec![ExprTerm::Arg(0), ExprTerm::Arg(1)],
        }),
        EExpr::Fn(FnName::Concat),
    ];
    let validators = vec![
        Validator::Object(ObjectValidator {
            expression_id: 0,
            key_src: vec![
                ArgSrc::ConstsTpl("sale.permissions_contract_id".into()),
                ArgSrc::User("sale.permissions_contract_id".into()),
            ],
        }),
        Validator::Collection(CollectionValidator {
            prefixes: vec!["sale.out_tokens".into()],
            expression_id: 1,
            key_src: vec![
                ArgSrc::ConstsTpl("sale.out_tokens.balance".into()),
                ArgSrc::User("balance".into()),
            ],
        }),
        Validator::Collection(CollectionValidator {
            prefixes: vec!["sale.out_tokens".into()],
            expression_id: 1,
            key_src: vec![
                ArgSrc::ConstsTpl("sale.out_tokens.shares.amount".into()),
                ArgSrc::User("shares.amount".into()),
            ],
        }),
    ];

    let binds: Vec<BindDefinition> = vec![
        BindDefinition {
            key: "sale.meta.reason".into(),
            key_src: SrcOrExpr::Src(ArgSrc::ConstsTpl("sale.meta.reason".into())),
            is_collection: false,
            prefixes: vec![],
        },
        BindDefinition {
            key: "sale_info".into(),
            key_src: SrcOrExpr::Src(ArgSrc::ConstsTpl("sale_info".into())),
            is_collection: false,
            prefixes: vec![],
        },
        BindDefinition {
            key: "token_account_id".into(),
            key_src: SrcOrExpr::Expr(Expression {
                args: vec![ArgSrc::ConstsTpl("sale.out_tokens.token_account_id".into())],
                expr_id: 2,
            }),
            is_collection: true,
            prefixes: vec!["sale.out_tokens".into()],
        },
    ];

    TestCase {
        fncall_metadata,
        validators,
        expressions,
        binds,
        tpl_consts,
    }
}
fn input_skyward() -> UserInput {
    let mut hm = InputHashMap::new();

    // Object 0
    hm.set("sale_info", Value::String("Sale info from user".into()));

    // Object 1
    hm.set("sale.url", Value::String("www.neardao.com".into()));
    hm.set("sale.title", Value::String("Neardao token auction".into()));
    hm.set(
        "sale.permissions_contract_id",
        Value::String("neardao.testnet".into()),
    );
    hm.set(
        "sale.in_token_account_id",
        Value::String("wrap.near".into()),
    );
    hm.set("sale.start_time", Value::U128(0.into()));
    hm.set("sale.duration", Value::U128(3600.into()));

    // Object 2
    hm.set(
        "sale.meta.reason",
        Value::String("Meta reason from user".into()),
    );
    hm.set("sale.meta.timestamp", Value::U64(420));

    // Object 3 + 4
    hm.set(
        "sale.out_tokens.0.token_account_id",
        Value::String("neardao.testnet".into()),
    );
    hm.set("sale.out_tokens.0.balance", Value::U128(999.into()));
    hm.set("sale.out_tokens.0.referral_bpt", Value::U64(420));
    hm.set(
        "sale.out_tokens.0.shares.user",
        Value::String("petr.near".into()),
    );
    hm.set("sale.out_tokens.0.shares.amount", Value::U64(123));

    hm.set(
        "sale.out_tokens.1.token_account_id",
        Value::String("neardao.testnet".into()),
    );
    hm.set("sale.out_tokens.1.balance", Value::U128(991.into()));
    hm.set("sale.out_tokens.1.referral_bpt", Value::Null);
    hm.set(
        "sale.out_tokens.1.shares.user",
        Value::String("david.near".into()),
    );
    hm.set("sale.out_tokens.1.shares.amount", Value::U64(456));
    hm.set(
        "sale.out_tokens.2.token_account_id",
        Value::String("neardao.testnet".into()),
    );
    hm.set("sale.out_tokens.2.balance", Value::U128(991.into()));
    hm.set("sale.out_tokens.2.referral_bpt", Value::U64(420));
    hm.set(
        "sale.out_tokens.2.shares.user",
        Value::String("tomas.near".into()),
    );
    hm.set("sale.out_tokens.2.shares.amount", Value::U64(456));

    UserInput::Map(hm)
}
