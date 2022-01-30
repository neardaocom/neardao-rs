use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::UnorderedMap,
    env,
    serde::Serialize,
    IntoStorageKey,
};

use crate::errors::ERR_STORAGE_INVALID_TYPE;


#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum DataType {
    String,
    Bool,
    U8,
    U16,
    U32,
    U64,
    U128,
    VecString,
    VecBool,
    VecU8,
    VecU16,
    VecU32,
    VecU64,
    VecU128,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct StorageBucket {
    pub data: UnorderedMap<String, StorageData>,
}

impl StorageBucket {
    pub fn new<T: IntoStorageKey>(storage_key: T) -> Self {
        StorageBucket {
            data: UnorderedMap::new(storage_key.into_storage_key()),
        }
    }

    pub fn get_all_data(&self) -> Vec<(String, StorageData)> {
        self.data.to_vec()
    }

    pub fn get_data(&self, key: &String) -> Option<StorageData> {
        self.data.get(key)
    }

    pub fn add_data(&mut self, key: &String, data: &StorageData) {
        self.data.insert(key, data);
    }

    pub fn remove_data(&mut self, key: &String) -> Option<StorageData> {
        self.data.remove(key)
    }

    pub fn remove_storage_data(&mut self) {
        self.data.clear();
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct StorageData {
    pub datatype: DataType,
    pub data: Vec<u8>,
}

macro_rules! impl_into_type {
    ($t:ty, $variant:ident, $fn_name:ident) => {
        pub fn $fn_name(self) -> Result<$t, String> {
            match self.datatype {
                DataType::$variant => Ok(BorshDeserialize::try_from_slice(&self.data).unwrap()),
                _ => Err(ERR_STORAGE_INVALID_TYPE.into()),
            }
        }
    };
}

impl StorageData {
    impl_into_type!(String, String, try_into_string);
    impl_into_type!(bool, Bool, try_into_bool);
    impl_into_type!(u8, U8, try_into_u8);
    impl_into_type!(u16, U16, try_into_u16);
    impl_into_type!(u32, U32, try_into_u32);
    impl_into_type!(u64, U64, try_into_u64);
    impl_into_type!(u128, U128, try_into_u128);
    impl_into_type!(Vec<String>, VecString, try_into_vec_string);
    impl_into_type!(Vec<bool>, VecBool, try_into_vec_bool);
    impl_into_type!(Vec<u8>, VecU8, try_into_vec_u8);
    impl_into_type!(Vec<u16>, VecU16, try_into_vec_u16);
    impl_into_type!(Vec<u32>, VecU32, try_into_vec_u32);
    impl_into_type!(Vec<u64>, VecU64, try_into_vec_u64);
    impl_into_type!(Vec<u128>, VecU128, try_into_vec_u128);
}

macro_rules! impl_from {
    ($from:ty, $type_variant:ident) => {
        impl From<$from> for StorageData {
            fn from(input: $from) -> Self {
                StorageData {
                    datatype: DataType::$type_variant,
                    data: input.try_to_vec().unwrap(),
                }
            }
        }
    };
}

impl_from!(bool, Bool);
impl_from!(String, String);
impl_from!(u8, U8);
impl_from!(u16, U16);
impl_from!(u32, U32);
impl_from!(u64, U64);
impl_from!(u128, U128);
impl_from!(Vec<String>, VecString);
impl_from!(Vec<bool>, VecBool);
impl_from!(Vec<u8>, VecU8);
impl_from!(Vec<u16>, VecU16);
impl_from!(Vec<u32>, VecU32);
impl_from!(Vec<u64>, VecU64);
impl_from!(Vec<u128>, VecU128);
