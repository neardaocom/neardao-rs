use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::UnorderedMap,
    IntoStorageKey,
};

use crate::types::datatype::Value;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct StorageBucket {
    data: UnorderedMap<String, Value>,
}

impl StorageBucket {
    pub fn new<T: IntoStorageKey>(storage_key: T) -> Self {
        StorageBucket {
            data: UnorderedMap::new(storage_key.into_storage_key()),
        }
    }

    pub fn get_all_data(&self) -> Vec<(String, Value)> {
        self.data.to_vec()
    }

    #[allow(clippy::ptr_arg)]
    pub fn get_data(&self, key: &String) -> Option<Value> {
        self.data.get(key)
    }

    #[allow(clippy::ptr_arg)]
    pub fn add_data(&mut self, key: &String, data: &Value) {
        self.data.insert(key, data);
    }

    #[allow(clippy::ptr_arg)]
    pub fn remove_data(&mut self, key: &String) -> Option<Value> {
        self.data.remove(key)
    }

    pub fn remove_storage_data(&mut self) {
        self.data.clear();
    }
}
