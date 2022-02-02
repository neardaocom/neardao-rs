use near_sdk::serde_json;
use near_sdk::{env, ext_contract, near_bindgen, PromiseResult};

use crate::action::DataTypeDef;
use crate::core::*;
use crate::errors::*;
use crate::storage::DataType;
use crate::workflow::Postprocessing;

#[ext_contract(ext_self)]
trait ExtSelf {
    fn postprocess(&mut self, storage_key: String, postprocessing: Postprocessing) -> u32;
    //fn callback_insert_skyward_auction(&mut self) -> u64;
}

#[near_bindgen]
impl NewDaoContract {
    // TODO error handling
    #[private]
    pub fn postprocess(&mut self, storage_key: String, postprocessing: &Postprocessing) {
        assert_eq!(
            env::promise_results_count(),
            1,
            "{}",
            ERR_PROMISE_INVALID_RESULTS_COUNT
        );
        match env::promise_result(0) {
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
            }
            PromiseResult::Failed => env::panic(ERR_PROMISE_FAILED.as_bytes()),
        }
    }
}
