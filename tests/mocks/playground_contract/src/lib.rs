#![allow(unused)]
#![allow(clippy::all)]

use std::collections::HashMap;
use std::hash::Hash;
use std::{unimplemented, vec::Vec};

use library::functions::utils::calculate_percent_u128;
use library::functions::utils::into_storage_key_wrapper_u16;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, log, near_bindgen, BorshStorageKey, PanicOnDefault};
use types::{PercentInput, PercentResult, SourceMock};

const NESTED_PREFIX: &[u8; 3] = b"grp";

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    One,
    Two,
    Three,
    HashMapMaxSizeTest,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    pub int_key_lm: LookupMap<u16, String>,
    pub key_1: u16,
    pub int_key_hm: LazyOption<HashMap<u16, String>>,
    pub key_2: u16,
    pub int_nested_lm: LookupMap<u16, LookupMap<u16, String>>,
    pub key_3: u16,
    pub hm_max_size_test: LazyOption<HashMap<String, u8>>,
}

impl Default for Contract {
    fn default() -> Self {
        Self::new()
    }
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        let contract = Self {
            int_key_lm: LookupMap::new(StorageKeys::One),
            int_key_hm: LazyOption::new(StorageKeys::Two, Some(&HashMap::new())),
            int_nested_lm: LookupMap::new(StorageKeys::Three),
            key_1: 0,
            key_2: 0,
            key_3: 0,
            hm_max_size_test: LazyOption::new(
                StorageKeys::HashMapMaxSizeTest,
                Some(&HashMap::new()),
            ),
        };
        contract
    }

    pub fn add_to_lm(&mut self, values: Vec<String>) {
        for v in values {
            self.key_1 += 1;
            let error_msg = format!("lm - collision at {}", self.key_1);
            assert!(
                self.int_key_lm.insert(&self.key_1, &v).is_none(),
                "{}",
                &error_msg
            );
        }
    }

    pub fn add_to_hm(&mut self, values: Vec<String>) {
        let mut hm = self.int_key_hm.get().unwrap();
        for v in values {
            self.key_2 += 1;
            let error_msg = format!("hm - collision at {}", self.key_2);
            assert!(hm.insert(self.key_2, v).is_none(), "{}", &error_msg);
        }
        self.int_key_hm.set(&hm);
    }

    pub fn add_to_nested(&mut self, values: Vec<String>) {
        for (i, v) in values.into_iter().enumerate() {
            let i = i as u16;
            self.key_3 += 1;
            let key = into_storage_key_wrapper_u16(NESTED_PREFIX, self.key_3);
            let mut map = LookupMap::new(key);
            assert!(map.insert(&(i), &v).is_none(), "Nested collision");
            let error_msg = format!("hm - collision at {}", self.key_3);
            assert!(
                self.int_nested_lm.insert(&i, &map).is_none(),
                "{}",
                &error_msg
            );
        }
    }

    pub fn add_to_hm_size_test(&mut self, values: Vec<(String, u8)>) {
        let init_size = env::storage_usage();
        let mut hm = self.hm_max_size_test.get().unwrap();
        for (acc, vote) in values {
            hm.insert(acc, vote);
        }
        self.hm_max_size_test.set(&hm);
        log!("storage usage diff: +{}", env::storage_usage() - init_size);
    }

    pub fn view_hm_size(&self) -> usize {
        self.hm_max_size_test.get().unwrap().len()
    }
    pub fn view_hm(&self) -> Vec<(String, u8)> {
        self.hm_max_size_test.get().unwrap().into_iter().collect()
    }
    pub fn calc_votes(&mut self, values: Vec<PercentInput>) -> Vec<PercentResult> {
        let mut results = vec![];
        for value in values {
            let result = calculate_percent_u128(
                value.actual.0 * 10u128.pow(value.decimals as u32),
                value.total_possible.0 * 10u128.pow(value.decimals as u32),
            );
            results.push(PercentResult {
                total_possible: value.total_possible.into(),
                actual: value.actual.into(),
                decimals: value.decimals,
                result,
            })
        }
        results
    }
}
