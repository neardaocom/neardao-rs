use library::types::FnCallMetadata;
use library::workflow::{ActivityResult, Template, TemplateSettings};
use library::FnCallId;
use near_sdk::json_types::{U128, U64};
use near_sdk::serde_json;
use near_sdk::{env, ext_contract, near_bindgen, PromiseResult};

use crate::core::*;
use crate::errors::*;
use library::{
    types::{DataType, DataTypeDef},
    workflow::Postprocessing,
};

#[ext_contract(ext_self)]
trait ExtSelf {
    fn postprocess(
        &mut self,
        instance_id: u32,
        storage_key: String,
        postprocessing: Option<Postprocessing>,
    ) -> ActivityResult;

    fn store_workflow(
        &mut self,
        instance_id: u32,
        settings: Vec<TemplateSettings>,
    ) -> ActivityResult;
}

#[near_bindgen]
impl Contract {
    // TODO error handling
    #[private]
    pub fn postprocess(
        &mut self,
        instance_id: u32,
        storage_key: String,
        postprocessing: Option<Postprocessing>,
    ) -> ActivityResult {
        assert_eq!(
            env::promise_results_count(),
            1,
            "{}",
            ERR_PROMISE_INVALID_RESULTS_COUNT
        );
        let result: bool = match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(val) => match postprocessing {
                Some(p) => {
                    let value = match p.fn_call_result_type {
                        DataTypeDef::String(_) => {
                            DataType::String(serde_json::from_slice::<String>(&val).unwrap())
                        }
                        DataTypeDef::Bool(_) => {
                            DataType::Bool(serde_json::from_slice::<bool>(&val).unwrap())
                        }
                        DataTypeDef::U8(_) => {
                            DataType::U8(serde_json::from_slice::<u8>(&val).unwrap())
                        }
                        DataTypeDef::U16(_) => {
                            DataType::U16(serde_json::from_slice::<u16>(&val).unwrap())
                        }
                        DataTypeDef::U32(_) => {
                            DataType::U32(serde_json::from_slice::<u32>(&val).unwrap())
                        }
                        DataTypeDef::U64(_) => {
                            DataType::U64(serde_json::from_slice::<U64>(&val).unwrap())
                        }
                        DataTypeDef::U128(_) => {
                            DataType::U128(serde_json::from_slice::<U128>(&val).unwrap())
                        }
                        DataTypeDef::VecString => DataType::VecString(
                            serde_json::from_slice::<Vec<String>>(&val).unwrap(),
                        ),
                        DataTypeDef::VecU8 => {
                            DataType::VecU8(serde_json::from_slice::<Vec<u8>>(&val).unwrap())
                        }
                        DataTypeDef::VecU16 => {
                            DataType::VecU16(serde_json::from_slice::<Vec<u16>>(&val).unwrap())
                        }
                        DataTypeDef::VecU32 => {
                            DataType::VecU32(serde_json::from_slice::<Vec<u32>>(&val).unwrap())
                        }
                        DataTypeDef::VecU64 => {
                            DataType::VecU64(serde_json::from_slice::<Vec<U64>>(&val).unwrap())
                        }
                        DataTypeDef::VecU128 => {
                            DataType::VecU128(serde_json::from_slice::<Vec<U128>>(&val).unwrap())
                        }
                        DataTypeDef::Object(_) => {
                            unimplemented!("object is not supported yet");
                        }
                        DataTypeDef::NullableObject(_) => {
                            unimplemented!("object is not supported yet");
                        }
                        DataTypeDef::VecObject(_) => {
                            unimplemented!("object is not supported yet");
                        }
                    };

                    let mut bucket = self.storage.get(&storage_key).unwrap();
                    bucket.add_data(&p.storage_key, &value);
                    self.storage.insert(&storage_key, &bucket);
                    true
                }
                None => true,
            },
            PromiseResult::Failed => false,
        };

        match result {
            true => ActivityResult::Ok,
            false => self.postprocessing_fail_update(instance_id),
        }
    }

    #[private]
    pub fn store_workflow(
        &mut self,
        instance_id: u32,
        settings: Vec<TemplateSettings>,
    ) -> ActivityResult {
        assert_eq!(
            env::promise_results_count(),
            1,
            "{}",
            ERR_PROMISE_INVALID_RESULTS_COUNT
        );

        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(val) => {
                let (workflow, fncalls, fncall_metadata): (
                    Template,
                    Vec<FnCallId>,
                    Vec<Vec<FnCallMetadata>>,
                ) = serde_json::from_slice(&val).unwrap();
                self.workflow_last_id += 1;
                self.workflow_template
                    .insert(&self.workflow_last_id, &(workflow, settings));
                self.init_function_calls(fncalls, fncall_metadata);
                ActivityResult::Ok
            }
            PromiseResult::Failed => self.postprocessing_fail_update(instance_id),
        }
    }
}
