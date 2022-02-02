use library::workflow::{ActivityResult, InstanceState, Template};
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
        postprocessing: Postprocessing,
    ) -> ActivityResult;

    fn store_workflow(&mut self);
}

#[near_bindgen]
impl Contract {
    // TODO error handling
    #[private]
    pub fn postprocess(
        &mut self,
        instance_id: u32,
        storage_key: String,
        postprocessing: Postprocessing,
    ) -> ActivityResult {
        assert_eq!(
            env::promise_results_count(),
            1,
            "{}",
            ERR_PROMISE_INVALID_RESULTS_COUNT
        );
        let result = match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(val) => {
                let value: DataType = match postprocessing.fn_call_result_type {
                    DataTypeDef::String(_) => {
                        DataType::String(serde_json::from_slice::<String>(&val).unwrap())
                    }
                    DataTypeDef::Bool(_) => {
                        DataType::Bool(serde_json::from_slice::<bool>(&val).unwrap())
                    }
                    DataTypeDef::U8(_) => DataType::U8(serde_json::from_slice::<u8>(&val).unwrap()),
                    DataTypeDef::U16(_) => {
                        DataType::U16(serde_json::from_slice::<u16>(&val).unwrap())
                    }
                    DataTypeDef::U32(_) => {
                        DataType::U32(serde_json::from_slice::<u32>(&val).unwrap())
                    }
                    DataTypeDef::U64(_) => {
                        DataType::U64(serde_json::from_slice::<u64>(&val).unwrap())
                    }
                    DataTypeDef::U128(_) => {
                        DataType::U128(serde_json::from_slice::<u128>(&val).unwrap())
                    }
                    DataTypeDef::VecString => {
                        DataType::VecString(serde_json::from_slice::<Vec<String>>(&val).unwrap())
                    }
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
                        DataType::VecU64(serde_json::from_slice::<Vec<u64>>(&val).unwrap())
                    }
                    DataTypeDef::VecU128 => {
                        DataType::VecU128(serde_json::from_slice::<Vec<u128>>(&val).unwrap())
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
                bucket.add_data(&postprocessing.storage_key, &value);
                self.storage.insert(&storage_key, &bucket);
                true
            }
            PromiseResult::Failed => false,
        };

        match result {
            true => ActivityResult::Ok,
            false => {
                let mut wfi = self.workflow_instance.get(&instance_id).unwrap();
                wfi.current_activity_id -= 1;
                wfi.state = InstanceState::FatalError;
                self.workflow_instance.insert(&instance_id, &wfi).unwrap();
                ActivityResult::ErrPostprocessing
            }
        }
    }

    #[private]
    pub fn store_workflow(&mut self) {
        assert_eq!(
            env::promise_results_count(),
            1,
            "{}",
            ERR_PROMISE_INVALID_RESULTS_COUNT
        );

        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(val) => {
                let workflow: Template = serde_json::from_slice(&val).unwrap();
                self.workflow_last_id += 1;
                self.workflow_template
                    .insert(&self.workflow_last_id, &(workflow, vec![]));
            }
            PromiseResult::Failed => panic!("Failed to store workflow template"),
        }
    }
}
