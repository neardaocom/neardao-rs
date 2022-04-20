use std::borrow::{Borrow, BorrowMut};
use std::{unimplemented, vec::Vec};

use library::functions::binding::{bind_from_sources, bind_from_sources_new};
use library::functions::serialization::{serialize_to_json, serialize_to_json_new};
use library::functions::validation::{validate, validate_new};
use library::storage::StorageBucket;
use library::types::activity_input::{ActivityInput, ValueCollection};
use library::types::datatype::Value;
use library::types::source::Source;
use library::workflow::expression::{Expression, ExpressionNew};
use library::workflow::types::{ArgSrc, ArgSrcNew, FnCallMetadata, ValidatorRef, ValueContainer};
use library::{Consts, ObjectValues};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, PanicOnDefault};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub testcases_old: LookupMap<String, TestCaseOld>,
    pub testcases_new: LookupMap<String, TestCaseNew>,
    pub global_storage: StorageBucket,
    pub called: u16,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        testcases_old: Vec<(String, TestCaseOld)>,
        testcases_new: Vec<(String, TestCaseNew)>,
    ) -> Self {
        let mut contract = Self {
            testcases_old: LookupMap::new(b"told".to_vec()),
            testcases_new: LookupMap::new(b"tnew".to_vec()),
            global_storage: StorageBucket::new(b"global_storage".to_vec()),
            called: 0,
        };

        for (key, case) in testcases_old.into_iter() {
            contract.testcases_old.insert(&key, &case);
        }

        for (key, case) in testcases_new.into_iter() {
            contract.testcases_new.insert(&key, &case);
        }

        contract
    }

    pub fn bench_wf_old(&mut self, testcase: String, input: &mut ObjectValues) -> String {
        self.called += 1;

        let TestCaseOld {
            tpl_consts,
            fncall_metadata,
            validator_refs,
            validators,
            source_defs,
            expressions,
            ..
        } = self
            .testcases_old
            .get(&testcase)
            .expect("Testcase not found");

        let sources = ValueContainer {
            dao_consts: &self.get_dao_consts(),
            tpl_consts: &tpl_consts,
            settings_consts: &vec![],
            activity_shared_consts: None,
            action_proposal_consts: None,
            storage: None,
            global_storage: &mut self.global_storage,
        };

        let _validation_result = validate(
            &sources,
            validator_refs.as_slice(),
            validators.as_slice(),
            fncall_metadata.as_slice(),
            input.as_slice(),
        )
        .expect("Validation failed");
        let _bind_result = bind_from_sources(
            source_defs.as_slice(),
            &sources,
            expressions.as_slice(),
            input,
            0,
        )
        .expect("Binding failed");

        serialize_to_json(input.as_slice(), fncall_metadata.as_slice(), 0)
    }

    pub fn bench_wf_new(&mut self, testcase: String, input: ValueCollection) -> String {
        self.called += 1;

        let mut user_input = input.into_activity_input();

        let TestCaseNew {
            tpl_consts,
            fncall_metadata,
            validator_refs,
            validators,
            source_defs,
            expressions,
            ..
        } = self
            .testcases_new
            .get(&testcase)
            .expect("Testcase not found");

        let sources = SourceMock { tpls: tpl_consts };

        let _validation_result = validate_new(
            &sources,
            validator_refs.as_slice(),
            validators.as_slice(),
            //fncall_metadata.as_slice(),
            user_input.borrow() as &dyn ActivityInput,
        )
        .expect("Validation failed");

        let _bind_result = bind_from_sources_new(
            &sources,
            source_defs.as_slice(),
            expressions.as_slice(),
            user_input.borrow_mut() as &mut dyn ActivityInput,
        )
        .expect("Binding failed");

        serialize_to_json_new(user_input, fncall_metadata.as_slice())
    }

    fn get_dao_consts(&self) -> Box<Consts> {
        Box::new(|id| match id {
            0 => Some(Value::String(env::current_account_id().to_string())),
            _ => unimplemented!(),
        })
    }
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TestCaseOld {
    pub fncall_metadata: Vec<FnCallMetadata>,
    pub validator_refs: Vec<ValidatorRef>,
    pub validators: Vec<Expression>,
    pub expressions: Vec<Expression>,
    pub source_defs: Vec<Vec<ArgSrc>>,
    pub tpl_consts: Vec<Value>,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TestCaseNew {
    pub fncall_metadata: Vec<FnCallMetadata>,
    pub validator_refs: Vec<ValidatorRef>,
    pub validators: Vec<ExpressionNew>,
    pub expressions: Vec<ExpressionNew>,
    pub source_defs: Vec<(String, ArgSrcNew)>,
    pub tpl_consts: Vec<(String, Value)>,
}

pub struct SourceMock {
    tpls: Vec<(String, Value)>,
}

impl Source for SourceMock {
    fn get_tpl_const(&self, key: &str) -> Option<&Value> {
        self.tpls.iter().find(|el| el.0 == key).map(|el| &el.1)
    }
}
