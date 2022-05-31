#![cfg(test)]
#![allow(unused)]

use std::{collections::HashMap, convert::TryFrom};

use near_sdk::{
    json_types::U128,
    serde::{Deserialize, Serialize},
    serde_json, AccountId,
};

use crate::{
    functions::{binding::bind_input, serialization::serialize_to_json, validation::validate},
    interpreter::expression::{EExpr, EOp, ExprTerm, FnName, Op, RelOp, TExpr},
    storage::StorageBucket,
    types::{
        activity_input::ActivityInput,
        datatype::{Datatype, Value},
        source::{SourceMock, SourceProvider},
    },
    workflow::{
        expression::Expression,
        types::{
            BindDefinition, CollectionBindData, CollectionBindingStyle, ObjectMetadata, Src,
            ValueSrc,
        },
        validator::{CollectionValidator, ObjectValidator, Validator},
    },
};

/******  Skyward sale_create structures  ******/

type BasicPoints = u16;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct SaleCreateInput {
    pub sale: SaleInput,
    pub sale_info: Option<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct SaleInput {
    pub title: String,
    pub url: Option<String>,
    pub permissions_contract_id: Option<AccountId>,
    pub out_tokens: Vec<SaleInputOutToken>,
    pub in_token_account_id: AccountId,
    pub start_time: U128,
    pub duration: U128,
    pub meta: MetaInfo,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct MetaInfo {
    reason: String,
    timestamp: u64,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct SaleInputOutToken {
    pub token_account_id: AccountId,
    pub balance: U128,
    pub referral_bpt: Option<BasicPoints>,
    pub shares: ShareInfo,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct ShareInfo {
    user: String,
    amount: u32,
}
/****** TEST CASES ******/

#[test]
fn full_scenario_skyward_validation_binding_serialization_complex() {
    let metadata = vec![
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
    let source = SourceMock { tpls: tpl_consts };
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
            value: vec![
                ValueSrc::Src(Src::Tpl("sale.permissions_contract_id".into())),
                ValueSrc::Src(Src::Input("sale.permissions_contract_id".into())),
            ],
        }),
        Validator::Collection(CollectionValidator {
            prefixes: vec!["sale.out_tokens".into()],
            expression_id: 1,
            value: vec![
                ValueSrc::Src(Src::Tpl("sale.out_tokens.balance".into())),
                ValueSrc::Src(Src::Input("balance".into())),
            ],
        }),
        Validator::Collection(CollectionValidator {
            prefixes: vec!["sale.out_tokens".into()],
            expression_id: 1,
            value: vec![
                ValueSrc::Src(Src::Tpl("sale.out_tokens.shares.amount".into())),
                ValueSrc::Src(Src::Input("shares.amount".into())),
            ],
        }),
    ];

    let mut hm = HashMap::new();

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
    let mut user_input: Box<dyn ActivityInput> = Box::new(hm);

    let mut global_storage = StorageBucket::new(b"global".to_vec());
    let activity_shared_consts: Vec<Value> = vec![];

    let dao_consts = Box::new(|id: u8| match id {
        0 => Some(Value::String("neardao.near".into())),
        _ => None,
    });

    assert!(validate(
        &source,
        validators.as_slice(),
        expressions.as_slice(),
        user_input.as_ref(),
    )
    .expect("Validation failed."));

    /* ------------------ Binding ------------------ */

    let source_defs: Vec<BindDefinition> = vec![
        BindDefinition {
            key: "sale.meta.reason".into(),
            value: ValueSrc::Src(Src::Tpl("sale.meta.reason".into())),
            collection_data: None,
        },
        BindDefinition {
            key: "sale_info".into(),
            value: ValueSrc::Src(Src::Tpl("sale_info".into())),
            collection_data: None,
        },
        BindDefinition {
            key: "token_account_id".into(),
            value: ValueSrc::Expr(Expression {
                args: vec![Src::Tpl("sale.out_tokens.token_account_id".into())],
                expr_id: 2,
            }),
            collection_data: Some(CollectionBindData {
                prefixes: vec!["sale.out_tokens".into()],
                collection_binding_type: CollectionBindingStyle::Overwrite,
            }),
        },
    ];

    // Create expected bind result.
    let mut hm = HashMap::new();
    // Object 0
    hm.set("sale_info", Value::String("sale info binded".into()));

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
        Value::String("meta reason binded".into()),
    );
    hm.set("sale.meta.timestamp", Value::U64(420));

    // Object 3 + 4
    hm.set(
        "sale.out_tokens.0.token_account_id",
        Value::String("neardao.near".into()),
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
        Value::String("neardao.near".into()),
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
        Value::String("neardao.near".into()),
    );
    hm.set("sale.out_tokens.2.balance", Value::U128(991.into()));
    hm.set("sale.out_tokens.2.referral_bpt", Value::U64(420));
    hm.set(
        "sale.out_tokens.2.shares.user",
        Value::String("tomas.near".into()),
    );
    hm.set("sale.out_tokens.2.shares.amount", Value::U64(456));
    let expected: Box<dyn ActivityInput> = Box::new(hm);

    bind_input(&source, &source_defs, &expressions, user_input.as_mut()).expect("Binding failed");

    let mut actual_user_input = user_input.to_vec();
    let mut expected_user_input = expected.to_vec();
    actual_user_input.sort_by(|a, b| a.0.cmp(&b.0));
    expected_user_input.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(actual_user_input, expected_user_input,);

    /* ------------------ Serializing to JSON ------------------ */

    let out_tokens_1 = SaleInputOutToken {
        token_account_id: AccountId::try_from("neardao.near".to_string()).unwrap(),
        balance: 999.into(),
        referral_bpt: Some(420),
        shares: ShareInfo {
            user: "petr.near".into(),
            amount: 123,
        },
    };

    let out_tokens_2 = SaleInputOutToken {
        token_account_id: AccountId::try_from("neardao.near".to_string()).unwrap(),
        balance: 991.into(),
        referral_bpt: None,
        shares: ShareInfo {
            user: "david.near".into(),
            amount: 456,
        },
    };

    let out_tokens_3 = SaleInputOutToken {
        token_account_id: AccountId::try_from("neardao.near".to_string()).unwrap(),
        balance: 991.into(),
        referral_bpt: Some(420),
        shares: ShareInfo {
            user: "tomas.near".into(),
            amount: 456,
        },
    };

    let sale_input = SaleInput {
        title: "Neardao token auction".into(),
        url: Some("www.neardao.com".into()),
        permissions_contract_id: Some(AccountId::try_from("neardao.testnet".to_string()).unwrap()),
        out_tokens: vec![out_tokens_1, out_tokens_2, out_tokens_3],
        in_token_account_id: AccountId::try_from("wrap.near".to_string()).unwrap(),
        start_time: 0.into(),
        duration: 3600.into(),
        meta: MetaInfo {
            reason: "meta reason binded".into(),
            timestamp: 420,
        },
    };

    let sale_create_input = SaleCreateInput {
        sale: sale_input,
        sale_info: Some("sale info binded".into()),
    };

    let result_json_string = serialize_to_json(user_input, metadata.as_slice()).unwrap();
    let expected_json_string = serde_json::to_string(&sale_create_input).unwrap();
    assert_eq!(result_json_string, expected_json_string);
    assert_eq!(
        serde_json::from_str::<SaleCreateInput>(&result_json_string).unwrap(),
        sale_create_input
    );
}

/* Test objects -- serialize_complex */

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
struct Job {
    name: String,
    started: u64,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
struct UserHobby {
    name: String,
    years_doing: u8,
    coach: Option<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
struct Partner {
    fullname: String,
    interested_in_crypto: Option<bool>,
    hobbies: Vec<UserHobby>,
}
#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
struct PersonInfo {
    first_name: String,
    surname: String,
    age: u8,
    car: Car,
    animals: Vec<Animals>,
    partner: Option<Partner>,
    job: Option<Job>,
    other: Option<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
struct Car {
    brand: String,
    model: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
struct Cage {
    size: u8,
    material: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
struct Animals {
    name: String,
    age: Option<u8>,
    cage: Option<Cage>,
}

/* Test objects */

#[test]
fn serialize_complex() {
    let metadata = vec![
        ObjectMetadata {
            arg_names: vec![
                "first_name".into(),
                "surname".into(),
                "age".into(),
                "car".into(),
                "animals".into(),
                "partner".into(),
                "job".into(),
                "other".into(),
            ],
            arg_types: vec![
                Datatype::String(false),
                Datatype::String(false),
                Datatype::U64(false),
                Datatype::Object(1),
                Datatype::VecObject(2),
                Datatype::OptionalObject(3),
                Datatype::OptionalObject(4),
                Datatype::String(true),
            ],
        },
        ObjectMetadata {
            arg_names: vec!["brand".into(), "model".into()],
            arg_types: vec![Datatype::String(false), Datatype::String(false)],
        },
        ObjectMetadata {
            arg_names: vec!["name".into(), "age".into(), "cage".into()],
            arg_types: vec![
                Datatype::String(false),
                Datatype::U64(true),
                Datatype::OptionalObject(6),
            ],
        },
        ObjectMetadata {
            arg_names: vec![
                "fullname".into(),
                "interested_in_crypto".into(),
                "hobbies".into(),
            ],
            arg_types: vec![
                Datatype::String(false),
                Datatype::Bool(false),
                Datatype::VecObject(5),
            ],
        },
        ObjectMetadata {
            arg_names: vec!["name".into(), "started".into()],
            arg_types: vec![Datatype::String(false), Datatype::U64(false)],
        },
        ObjectMetadata {
            arg_names: vec!["name".into(), "years_doing".into(), "coach".into()],
            arg_types: vec![
                Datatype::String(false),
                Datatype::U64(false),
                Datatype::String(true),
            ],
        },
        ObjectMetadata {
            arg_names: vec!["size".into(), "material".into()],
            arg_types: vec![Datatype::U64(false), Datatype::String(false)],
        },
    ];

    let mut input_data = HashMap::new();
    input_data.set("first_name", Value::String("Mike".into()));
    input_data.set("surname", Value::String("Segal".into()));
    input_data.set("age", Value::U64(25));
    input_data.set("car.brand", Value::String("ford".into()));
    input_data.set("car.model", Value::String("mustang".into()));
    input_data.set("animals.0.name", Value::String("Sandy".into()));
    input_data.set("animals.0.age", Value::U64(1));
    input_data.set("animals.0.cage.size", Value::U64(5));
    input_data.set("animals.0.cage.material", Value::String("wood".into()));
    input_data.set("animals.1.name", Value::String("Betty".into()));
    input_data.set("animals.1.cage", Value::Null);
    input_data.set("partner.fullname", Value::String("Video games".into()));
    input_data.set("partner.interested_in_crypto", Value::Bool(true));
    input_data.set("partner.hobbies.0.name", Value::String("Gambling".into()));
    input_data.set("partner.hobbies.0.years_doing", Value::U64(1));
    input_data.set(
        "partner.hobbies.1.name",
        Value::String("Video games".into()),
    );
    input_data.set("partner.hobbies.1.years_doing", Value::U64(1));
    input_data.set(
        "partner.hobbies.1.coach",
        Value::String("Ronnie Coleman".into()),
    );
    input_data.set("job", Value::Null);
    input_data.set("other", Value::String("Other info".into()));

    let input = Box::new(input_data);

    let json = serialize_to_json(input, metadata.as_slice()).unwrap();

    let _: PersonInfo =
        serde_json::from_str(&json).expect("Failed to deserialize person from string");
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
enum TestEnum {
    First(FirstStruct),
    Second(SecondStruct),
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct TestStruct {
    name: String,
    r#type: Option<TestEnum>,
    info: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
struct FirstStruct {
    name: String,
}
#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
struct SecondStruct {
    name: String,
}

#[test]
fn serialize_enum_1() {
    let metadata = vec![
        ObjectMetadata {
            arg_names: vec!["name".into(), "type".into(), "info".into()],
            arg_types: vec![
                Datatype::String(false),
                Datatype::OptionalEnum(vec![1, 2]),
                Datatype::String(false),
            ],
        },
        ObjectMetadata {
            arg_names: vec!["first.name".into()],
            arg_types: vec![Datatype::String(false)],
        },
        ObjectMetadata {
            arg_names: vec!["second.name".into()],
            arg_types: vec![Datatype::String(false)],
        },
    ];
    let mut input: HashMap<String, Value> = HashMap::new();
    input.insert("name".into(), Value::String("test".into()));
    input.insert("info".into(), Value::String("info".into()));
    input.insert(
        "type.first.name".into(),
        Value::String("first_struct_name".into()),
    );
    let input = Box::new(input);
    let actual_json = serialize_to_json(input, metadata.as_slice()).unwrap();
    let actual_obj: TestStruct =
        serde_json::from_str(&actual_json).expect("Failed to deserialize enum from string");
    let expected_obj = TestStruct {
        name: "test".into(),
        info: "info".into(),
        r#type: Some(TestEnum::First(FirstStruct {
            name: "first_struct_name".into(),
        })),
    };
    let expected_json = serde_json::to_string(&expected_obj).unwrap();
    assert_eq!(actual_json, expected_json);
    assert_eq!(actual_obj, expected_obj);
}

#[test]
fn serialize_enum_2() {
    let metadata = vec![
        ObjectMetadata {
            arg_names: vec!["name".into(), "type".into(), "info".into()],
            arg_types: vec![
                Datatype::String(false),
                Datatype::OptionalEnum(vec![1, 2]),
                Datatype::String(false),
            ],
        },
        ObjectMetadata {
            arg_names: vec!["first.name".into()],
            arg_types: vec![Datatype::String(false)],
        },
        ObjectMetadata {
            arg_names: vec!["second.name".into()],
            arg_types: vec![Datatype::String(false)],
        },
    ];
    let mut input: HashMap<String, Value> = HashMap::new();
    input.insert("name".into(), Value::String("test".into()));
    input.insert("info".into(), Value::String("info".into()));
    input.insert(
        "type.second.name".into(),
        Value::String("second_struct_name".into()),
    );
    let input = Box::new(input);
    let actual_json = serialize_to_json(input, metadata.as_slice()).unwrap();
    let actual_obj: TestStruct =
        serde_json::from_str(&actual_json).expect("Failed to deserialize enum from string");
    let expected_obj = TestStruct {
        name: "test".into(),
        info: "info".into(),
        r#type: Some(TestEnum::Second(SecondStruct {
            name: "second_struct_name".into(),
        })),
    };
    let expected_json = serde_json::to_string(&expected_obj).unwrap();
    assert_eq!(actual_json, expected_json);
    assert_eq!(actual_obj, expected_obj);
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
struct MixedStruct {
    name: String,
    r#type: MixedEnum,
    info: String,
}
#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
enum MixedEnum {
    Near,
    Ft(FirstStruct),
}

#[test]
fn serialize_enum_3() {
    let metadata = vec![
        ObjectMetadata {
            arg_names: vec!["name".into(), "type".into(), "info".into()],
            arg_types: vec![
                Datatype::String(false),
                Datatype::OptionalEnum(vec![1, 2]),
                Datatype::String(false),
            ],
        },
        ObjectMetadata {
            arg_names: vec!["near".into()],
            arg_types: vec![],
        },
        ObjectMetadata {
            arg_names: vec!["ft.name".into()],
            arg_types: vec![Datatype::String(false)],
        },
    ];
    let mut input: HashMap<String, Value> = HashMap::new();
    input.insert("name".into(), Value::String("test".into()));
    input.insert("info".into(), Value::String("info".into()));
    input.insert(
        "type.near".into(),
        Value::String("first_struct_name".into()),
    );
    let input = Box::new(input);
    let actual_json = serialize_to_json(input, metadata.as_slice()).unwrap();
    let actual_obj: MixedStruct =
        serde_json::from_str(&actual_json).expect("Failed to deserialize enum from string");
    let expected_obj = MixedStruct {
        name: "test".into(),
        info: "info".into(),
        r#type: MixedEnum::Near,
    };
    let expected_json = serde_json::to_string(&expected_obj).unwrap();
    assert_eq!(actual_json, expected_json);
    assert_eq!(actual_obj, expected_obj);

    let mut input: HashMap<String, Value> = HashMap::new();
    input.insert("name".into(), Value::String("test".into()));
    input.insert("info".into(), Value::String("info".into()));
    input.insert(
        "type.ft.name".into(),
        Value::String("first_struct_name".into()),
    );
    let input = Box::new(input);
    let actual_json = serialize_to_json(input, metadata.as_slice()).unwrap();
    let actual_obj: MixedStruct =
        serde_json::from_str(&actual_json).expect("Failed to deserialize enum from string");
    let expected_obj = MixedStruct {
        name: "test".into(),
        info: "info".into(),
        r#type: MixedEnum::Ft(FirstStruct {
            name: "first_struct_name".into(),
        }),
    };
    let expected_json = serde_json::to_string(&expected_obj).unwrap();
    assert_eq!(actual_json, expected_json);
    assert_eq!(actual_obj, expected_obj);
}

#[test]
fn serialize_enum_opional_missing() {
    let metadata = vec![
        ObjectMetadata {
            arg_names: vec!["name".into(), "type".into(), "info".into()],
            arg_types: vec![
                Datatype::String(false),
                Datatype::OptionalEnum(vec![1, 2]),
                Datatype::String(false),
            ],
        },
        ObjectMetadata {
            arg_names: vec!["first.name".into()],
            arg_types: vec![Datatype::String(false)],
        },
        ObjectMetadata {
            arg_names: vec!["second.name".into()],
            arg_types: vec![Datatype::String(false)],
        },
    ];
    let mut input: HashMap<String, Value> = HashMap::new();
    input.insert("name".into(), Value::String("test".into()));
    input.insert("info".into(), Value::String("info".into()));
    let input = Box::new(input);
    let actual_json = serialize_to_json(input, metadata.as_slice()).unwrap();
    let actual_obj: TestStruct =
        serde_json::from_str(&actual_json).expect("Failed to deserialize enum from string");
    let expected_obj = TestStruct {
        name: "test".into(),
        info: "info".into(),
        r#type: None,
    };
    let expected_json = serde_json::to_string(&expected_obj).unwrap();
    assert_eq!(actual_json, expected_json);
    assert_eq!(actual_obj, expected_obj);
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
struct TestEnumVec {
    name: String,
    values: Vec<TestEnum>,
    info: String,
}

#[test]
fn serialize_vec_enum() {
    let metadata = vec![
        ObjectMetadata {
            arg_names: vec!["name".into(), "values".into(), "info".into()],
            arg_types: vec![
                Datatype::String(false),
                Datatype::VecEnum(vec![1, 2]),
                Datatype::String(false),
            ],
        },
        ObjectMetadata {
            arg_names: vec!["first.name".into()],
            arg_types: vec![Datatype::String(false)],
        },
        ObjectMetadata {
            arg_names: vec!["second.name".into()],
            arg_types: vec![Datatype::String(false)],
        },
    ];
    let mut input: HashMap<String, Value> = HashMap::new();
    input.insert("name".into(), Value::String("test1".into()));
    input.insert("info".into(), Value::String("info1".into()));
    input.insert(
        "values.0.first.name".into(),
        Value::String("first_string".into()),
    );
    input.insert(
        "values.1.second.name".into(),
        Value::String("second_string".into()),
    );
    input.insert("values.2.second".into(), Value::String("invalid".into()));
    let input = Box::new(input);
    let actual_json = serialize_to_json(input, metadata.as_slice()).unwrap();
    let actual_obj: TestEnumVec =
        serde_json::from_str(&actual_json).expect("Failed to deserialize enum from string");
    let expected_obj = TestEnumVec {
        name: "test1".into(),
        info: "info1".into(),
        values: vec![
            TestEnum::First(FirstStruct {
                name: "first_string".into(),
            }),
            TestEnum::Second(SecondStruct {
                name: "second_string".into(),
            }),
        ],
    };
    let expected_json = serde_json::to_string(&expected_obj).unwrap();
    assert_eq!(actual_json, expected_json);
    assert_eq!(actual_obj, expected_obj);
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
struct MixedStruct2 {
    name: String,
    r#type: Vec<MixedEnum>,
    info: String,
}

#[test]
fn serialize_vec_enum_2() {
    let metadata = vec![
        ObjectMetadata {
            arg_names: vec!["name".into(), "type".into(), "info".into()],
            arg_types: vec![
                Datatype::String(false),
                Datatype::VecEnum(vec![1, 2]),
                Datatype::String(false),
            ],
        },
        ObjectMetadata {
            arg_names: vec!["near".into()],
            arg_types: vec![],
        },
        ObjectMetadata {
            arg_names: vec!["ft.name".into()],
            arg_types: vec![Datatype::String(false)],
        },
    ];
    let mut input: HashMap<String, Value> = HashMap::new();
    input.insert("name".into(), Value::String("test1".into()));
    input.insert("info".into(), Value::String("info1".into()));
    input.insert(
        "type.0.ft.name".into(),
        Value::String("first_string".into()),
    );
    input.insert("type.1.near".into(), Value::Null);
    let input = Box::new(input);
    let actual_json = serialize_to_json(input, metadata.as_slice()).unwrap();
    let actual_obj: MixedStruct2 =
        serde_json::from_str(&actual_json).expect("Failed to deserialize enum from string");
    let expected_obj = MixedStruct2 {
        name: "test1".into(),
        info: "info1".into(),
        r#type: vec![
            MixedEnum::Ft(FirstStruct {
                name: "first_string".into(),
            }),
            MixedEnum::Near,
        ],
    };
    let expected_json = serde_json::to_string(&expected_obj).unwrap();
    assert_eq!(actual_json, expected_json);
    assert_eq!(actual_obj, expected_obj);
}

#[test]
fn serialize_vec_enum_empty() {
    let metadata = vec![
        ObjectMetadata {
            arg_names: vec!["name".into(), "values".into(), "info".into()],
            arg_types: vec![
                Datatype::String(false),
                Datatype::VecEnum(vec![1, 2]),
                Datatype::String(false),
            ],
        },
        ObjectMetadata {
            arg_names: vec!["first.name".into()],
            arg_types: vec![Datatype::String(false)],
        },
        ObjectMetadata {
            arg_names: vec!["second.name".into()],
            arg_types: vec![Datatype::String(false)],
        },
    ];
    let mut input: HashMap<String, Value> = HashMap::new();
    input.insert("name".into(), Value::String("test1".into()));
    input.insert("info".into(), Value::String("info1".into()));
    input.insert(
        "values.0.first".into(),
        Value::String("does_not_count_as_valid".into()),
    );
    let input = Box::new(input);
    let actual_json = serialize_to_json(input, metadata.as_slice()).unwrap();
    let actual_obj: TestEnumVec =
        serde_json::from_str(&actual_json).expect("Failed to deserialize enum from string");
    let expected_obj = TestEnumVec {
        name: "test1".into(),
        info: "info1".into(),
        values: vec![],
    };
    let expected_json = serde_json::to_string(&expected_obj).unwrap();
    assert_eq!(actual_json, expected_json);
    assert_eq!(actual_obj, expected_obj);
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
struct TestStruct2 {
    name: String,
    obj: Vec<(String, u64)>,
    info: String,
}

#[test]
fn serialize_vec_tuple() {
    let metadata = vec![
        ObjectMetadata {
            arg_names: vec!["name".into(), "obj".into(), "info".into()],
            arg_types: vec![
                Datatype::String(false),
                Datatype::VecTuple(1),
                Datatype::String(false),
            ],
        },
        ObjectMetadata {
            arg_names: vec![],
            arg_types: vec![Datatype::String(false), Datatype::U64(false)],
        },
    ];
    let mut input: HashMap<String, Value> = HashMap::new();
    input.insert("name".into(), Value::String("test".into()));
    input.insert("info".into(), Value::String("info".into()));
    input.insert("obj.0.0".into(), Value::String("string1".into()));
    input.insert("obj.0.1".into(), Value::U64(1));
    input.insert("obj.1.0".into(), Value::String("string2".into()));
    input.insert("obj.1.1".into(), Value::U64(2));
    let input = Box::new(input);
    let actual_json = serialize_to_json(input, metadata.as_slice()).unwrap();
    let actual_obj: TestStruct2 =
        serde_json::from_str(&actual_json).expect("Failed to deserialize enum from string");
    let expected_obj = TestStruct2 {
        name: "test".into(),
        obj: vec![("string1".into(), 1), ("string2".into(), 2)],
        info: "info".into(),
    };
    let expected_json = serde_json::to_string(&expected_obj).unwrap();
    assert_eq!(actual_json, expected_json);
    assert_eq!(actual_obj, expected_obj);
}

#[test]
fn serialize_vec_tuple_empty() {
    let metadata = vec![
        ObjectMetadata {
            arg_names: vec!["name".into(), "obj".into(), "info".into()],
            arg_types: vec![
                Datatype::String(false),
                Datatype::VecTuple(1),
                Datatype::String(false),
            ],
        },
        ObjectMetadata {
            arg_names: vec![],
            arg_types: vec![Datatype::String(false), Datatype::U64(false)],
        },
    ];
    let mut input: HashMap<String, Value> = HashMap::new();
    input.insert("name".into(), Value::String("test".into()));
    input.insert("info".into(), Value::String("info".into()));
    let input = Box::new(input);
    let actual_json = serialize_to_json(input, metadata.as_slice()).unwrap();
    let actual_obj: TestStruct2 =
        serde_json::from_str(&actual_json).expect("Failed to deserialize enum from string");
    let expected_obj = TestStruct2 {
        name: "test".into(),
        obj: vec![],
        info: "info".into(),
    };
    let expected_json = serde_json::to_string(&expected_obj).unwrap();
    assert_eq!(actual_json, expected_json);
    assert_eq!(actual_obj, expected_obj);
}

#[test]
fn serialize_vec_tuple_empty_2() {
    let metadata = vec![
        ObjectMetadata {
            arg_names: vec!["name".into(), "obj".into(), "info".into()],
            arg_types: vec![
                Datatype::String(false),
                Datatype::VecTuple(1),
                Datatype::String(false),
            ],
        },
        ObjectMetadata {
            arg_names: vec![],
            arg_types: vec![Datatype::String(false), Datatype::U64(false)],
        },
    ];
    let mut input: HashMap<String, Value> = HashMap::new();
    input.insert("name".into(), Value::String("test".into()));
    input.insert("info".into(), Value::String("info".into()));
    input.insert("obj".into(), Value::Null);
    let input = Box::new(input);
    let actual_json = serialize_to_json(input, metadata.as_slice()).unwrap();
    let actual_obj: TestStruct2 =
        serde_json::from_str(&actual_json).expect("Failed to deserialize enum from string");
    let expected_obj = TestStruct2 {
        name: "test".into(),
        obj: vec![],
        info: "info".into(),
    };
    let expected_json = serde_json::to_string(&expected_obj).unwrap();
    assert_eq!(actual_json, expected_json);
    assert_eq!(actual_obj, expected_obj);
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
struct TestStruct3 {
    name: String,
    obj: Option<(String, u64)>,
    info: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
struct TestStruct4 {
    name: String,
    obj: (String, u64),
    info: String,
}

#[test]
fn serialize_tuple() {
    let metadata = vec![
        ObjectMetadata {
            arg_names: vec!["name".into(), "obj".into(), "info".into()],
            arg_types: vec![
                Datatype::String(false),
                Datatype::OptionalTuple(1),
                Datatype::String(false),
            ],
        },
        ObjectMetadata {
            arg_names: vec![],
            arg_types: vec![Datatype::String(false), Datatype::U64(false)],
        },
    ];
    let metadata_2 = vec![
        ObjectMetadata {
            arg_names: vec!["name".into(), "obj".into(), "info".into()],
            arg_types: vec![
                Datatype::String(false),
                Datatype::Tuple(1),
                Datatype::String(false),
            ],
        },
        ObjectMetadata {
            arg_names: vec![],
            arg_types: vec![Datatype::String(false), Datatype::U64(false)],
        },
    ];
    let mut input: HashMap<String, Value> = HashMap::new();
    input.insert("name".into(), Value::String("test".into()));
    input.insert("info".into(), Value::String("info".into()));
    input.insert("obj.0".into(), Value::String("string1".into()));
    input.insert("obj.1".into(), Value::U64(1));
    let input = Box::new(input);
    let input_2 = input.clone();
    let actual_json = serialize_to_json(input, metadata.as_slice()).unwrap();
    let actual_obj: TestStruct3 =
        serde_json::from_str(&actual_json).expect("Failed to deserialize enum from string");
    let expected_obj = TestStruct3 {
        name: "test".into(),
        obj: Some(("string1".into(), 1)),
        info: "info".into(),
    };
    let expected_json = serde_json::to_string(&expected_obj).unwrap();
    assert_eq!(actual_json, expected_json);
    assert_eq!(actual_obj, expected_obj);
    let actual_json = serialize_to_json(input_2, metadata_2.as_slice()).unwrap();
    let actual_obj: TestStruct4 =
        serde_json::from_str(&actual_json).expect("Failed to deserialize enum from string");
    let expected_obj = TestStruct4 {
        name: "test".into(),
        obj: ("string1".into(), 1),
        info: "info".into(),
    };
    let expected_json = serde_json::to_string(&expected_obj).unwrap();
    assert_eq!(actual_json, expected_json);
    assert_eq!(actual_obj, expected_obj);
}

#[test]
fn serialize_tuple_optional_missing() {
    let metadata = vec![
        ObjectMetadata {
            arg_names: vec!["name".into(), "obj".into(), "info".into()],
            arg_types: vec![
                Datatype::String(false),
                Datatype::OptionalTuple(1),
                Datatype::String(false),
            ],
        },
        ObjectMetadata {
            arg_names: vec![],
            arg_types: vec![Datatype::String(false), Datatype::U64(false)],
        },
    ];
    let mut input: HashMap<String, Value> = HashMap::new();
    input.insert("name".into(), Value::String("test".into()));
    input.insert("info".into(), Value::String("info".into()));
    let input = Box::new(input);
    let actual_json = serialize_to_json(input, metadata.as_slice()).unwrap();
    let actual_obj: TestStruct3 =
        serde_json::from_str(&actual_json).expect("Failed to deserialize enum from string");
    let expected_obj = TestStruct3 {
        name: "test".into(),
        obj: None,
        info: "info".into(),
    };
    let expected_json = serde_json::to_string(&expected_obj).unwrap();
    assert_eq!(actual_json, expected_json);
    assert_eq!(actual_obj, expected_obj);
}
