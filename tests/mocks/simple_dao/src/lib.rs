#![allow(unused)]

use std::{unimplemented, vec::Vec};

use library::functions::binding::bind_input;
use library::functions::serialization::serialize_to_json;
use library::functions::validation::validate;
use library::interpreter::expression::EExpr;
use library::storage::StorageBucket;
use library::types::activity_input::UserInput;
use library::types::consts::RuntimeConstantProvider;
use library::types::datatype::Value;
use library::types::source::SourceProvider;
use library::workflow::types::{BindDefinition, ObjectMetadata};
use library::workflow::validator::Validator;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, log, near_bindgen, PanicOnDefault};
use types::SourceMock;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub testcases: LookupMap<String, TestCase>,
    pub global_storage: StorageBucket,
    pub called: u16,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(testcases: Vec<(String, TestCase)>) -> Self {
        let mut contract = Self {
            testcases: LookupMap::new(b"testcases".to_vec()),
            global_storage: StorageBucket::new(b"global_storage".to_vec()),
            called: 0,
        };
        let storage_before = env::storage_usage();
        for (key, case) in testcases.into_iter() {
            contract.testcases.insert(&key, &case);
        }
        log!(
            "storage increase: {}",
            env::storage_usage() - storage_before
        );
        contract
    }

    pub fn validate_bind_serialize(&mut self, testcase: String, input: UserInput) -> String {
        self.called += 1;

        let mut user_input = input.into_activity_input();

        let TestCase {
            tpl_consts,
            fncall_metadata,
            validators,
            binds,
            expressions,
            ..
        } = self.testcases.get(&testcase).expect("Testcase not found");

        let sources = SourceMock { tpls: tpl_consts };

        assert!(validate(
            &sources,
            validators.as_slice(),
            expressions.as_slice(),
            user_input.as_ref(),
        )
        .expect("Validation failed"));

        bind_input(
            &sources,
            binds.as_slice(),
            expressions.as_slice(),
            user_input.as_mut(),
        )
        .expect("Binding failed");

        serialize_to_json(user_input, fncall_metadata.as_slice()).unwrap()
    }

    fn get_dao_consts(&self) -> impl RuntimeConstantProvider {
        DaoConsts::default()
    }
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TestCase {
    pub fncall_metadata: Vec<ObjectMetadata>,
    pub validators: Vec<Validator>,
    pub expressions: Vec<EExpr>,
    pub binds: Vec<BindDefinition>,
    pub tpl_consts: Vec<(String, Value)>,
}

#[derive(Default)]
pub struct DaoConsts;

impl RuntimeConstantProvider for DaoConsts {
    fn get(&self, key: u8) -> Option<Value> {
        match key {
            0 => Some(Value::String(env::current_account_id().to_string())),
            _ => None,
        }
    }
}
