use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::UnorderedMap,
    serde::{Deserialize, Serialize},
    IntoStorageKey,
};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum DataType {
    String(String),
    Bool(bool),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    VecString(Vec<String>),
    VecBool(Vec<bool>),
    VecU8(Vec<u8>),
    VecU16(Vec<u16>),
    VecU32(Vec<u32>),
    VecU64(Vec<u64>),
    VecU128(Vec<u128>),
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct StorageBucket {
    data: UnorderedMap<String, DataType>,
}

impl StorageBucket {
    pub fn new<T: IntoStorageKey>(storage_key: T) -> Self {
        StorageBucket {
            data: UnorderedMap::new(storage_key.into_storage_key()),
        }
    }

    pub fn get_all_data(&self) -> Vec<(String, DataType)> {
        self.data.to_vec()
    }

    pub fn get_data(&self, key: &String) -> Option<DataType> {
        self.data.get(key)
    }

    pub fn add_data(&mut self, key: &String, data: &DataType) {
        self.data.insert(key, data);
    }

    pub fn remove_data(&mut self, key: &String) -> Option<DataType> {
        self.data.remove(key)
    }

    pub fn remove_storage_data(&mut self) {
        self.data.clear();
    }
}
