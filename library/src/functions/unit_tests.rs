#![cfg(test)]

#[cfg(test)]
mod test {

    use std::convert::TryFrom;

    use near_sdk::{
        json_types::U128,
        serde::{Deserialize, Serialize},
        serde_json, AccountId,
    };

    use crate::{
        functions::{
            binding::bind_from_sources, serialization::serialize_to_json, validation::validate,
        },
        interpreter::expression::{EExpr, EOp, ExprTerm, Op, RelOp, TExpr},
        storage::StorageBucket,
        types::datatype::{Datatype, Value},
        workflow::{
            expression::Expression,
            types::{ArgSrc, FnCallMetadata, ValidatorRef, ValidatorType, ValueContainer},
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

    // TODO: Test with nullable object.

    #[test]
    fn full_scenario_validation_binding_serialization_complex_1() {
        let metadata = vec![
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

        let tpl_consts = vec![
            Value::String("neardao.testnet".into()),
            Value::String("neardao.near".into()),
        ];
        let settings_consts = vec![Value::U128(U128::from(1000))];
        let proposal_consts = vec![
            Value::U64(500),
            Value::String("info binded".into()),
            Value::String("testing binded".into()),
        ];
        let expressions = vec![];
        let validator_refs = vec![
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
        let validators = vec![
            // validates first obj
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
            // validates first collection obj
            Expression {
                args: vec![ArgSrc::ConstsSettings(0), ArgSrc::User(1)],
                expr: EExpr::Boolean(TExpr {
                    operators: vec![Op {
                        op_type: EOp::Rel(RelOp::Gt),
                        operands_ids: [0, 1],
                    }],
                    terms: vec![ExprTerm::Arg(0), ExprTerm::Arg(1)],
                }),
            },
            // validates second collection obj
            Expression {
                args: vec![ArgSrc::ConstAction(0), ArgSrc::User(1)],
                expr: EExpr::Boolean(TExpr {
                    operators: vec![Op {
                        op_type: EOp::Rel(RelOp::Gt),
                        operands_ids: [0, 1],
                    }],
                    terms: vec![ExprTerm::Arg(0), ExprTerm::Arg(1)],
                }),
            },
        ];

        let mut user_input = vec![
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
                Value::U128(U128::from(999)), // 1000 is limit
                Value::Null,
                Value::Null,
                Value::String("neardao.testnet".into()),
                Value::U128(U128::from(991)), // 1000 is limit
                Value::Null,
                Value::Null,
                Value::String("neardao.testnet".into()),
                Value::U128(U128::from(991)), // 1000 is limit
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

        let mut global_storage = StorageBucket::new(b"global".to_vec());
        let activity_shared_consts: Vec<Value> = vec![];

        let dao_consts = Box::new(|id: u8| match id {
            0 => Some(Value::String("neardao.near".into())),
            _ => None,
        });

        let value_source = ValueContainer {
            dao_consts: &dao_consts,
            tpl_consts: &tpl_consts,
            settings_consts: &settings_consts,
            activity_shared_consts: Some(&activity_shared_consts),
            action_proposal_consts: Some(&proposal_consts),
            storage: None,
            global_storage: &mut global_storage,
        };

        assert!(validate(
            &value_source,
            validator_refs.as_slice(),
            validators.as_slice(),
            metadata.as_slice(),
            user_input.as_slice(),
        )
        .expect("Validation failed."));

        /* ------------------ Binding ------------------ */

        let source_metadata = vec![
            vec![ArgSrc::Object(1), ArgSrc::ConstAction(1)],
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
            vec![ArgSrc::ConstAction(2), ArgSrc::User(1)],
            vec![
                ArgSrc::ConstsTpl(1),
                ArgSrc::User(1),
                ArgSrc::User(2),
                ArgSrc::VecObject(4),
            ],
            vec![ArgSrc::User(0), ArgSrc::User(1)],
        ];

        bind_from_sources(
            &source_metadata,
            &value_source,
            &expressions,
            &mut user_input,
            0,
        )
        .expect("Binding failed");

        let expected_binded_inputs = vec![
            vec![Value::Null, Value::String("info binded".into())],
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
            vec![Value::String("testing binded".into()), Value::U64(420)],
            vec![
                Value::String("neardao.near".into()),
                Value::U128(U128::from(999)), // 1000 is limit
                Value::Null,
                Value::Null,
                Value::String("neardao.near".into()),
                Value::U128(U128::from(991)), // 1000 is limit
                Value::Null,
                Value::Null,
                Value::String("neardao.near".into()),
                Value::U128(U128::from(991)), // 1000 is limit
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

        assert_eq!(user_input, expected_binded_inputs);

        /* ------------------ Serializing to JSON ------------------ */

        let out_tokens_1 = SaleInputOutToken {
            token_account_id: AccountId::try_from("neardao.near".to_string()).unwrap(),
            balance: 999.into(),
            referral_bpt: None,
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
            referral_bpt: None,
            shares: ShareInfo {
                user: "tomas.near".into(),
                amount: 456,
            },
        };

        let sale_input = SaleInput {
            title: "Neardao token auction".into(),
            url: Some("www.neardao.com".into()),
            permissions_contract_id: Some(
                AccountId::try_from("neardao.testnet".to_string()).unwrap(),
            ),
            out_tokens: vec![out_tokens_1, out_tokens_2, out_tokens_3],
            in_token_account_id: AccountId::try_from("wrap.near".to_string()).unwrap(),
            start_time: 0.into(),
            duration: 3600.into(),
            meta: MetaInfo {
                reason: "testing binded".into(),
                timestamp: 420,
            },
        };

        let sale_create_input = SaleCreateInput {
            sale: sale_input,
            sale_info: Some("info binded".into()),
        };

        let result_json_string = serialize_to_json(user_input.as_slice(), metadata.as_slice(), 0);
        let expected_json_string = serde_json::to_string(&sale_create_input).unwrap();
        assert_eq!(result_json_string, expected_json_string);
        assert_eq!(
            serde_json::from_str::<SaleCreateInput>(&result_json_string).unwrap(),
            sale_create_input
        );
    }
}
