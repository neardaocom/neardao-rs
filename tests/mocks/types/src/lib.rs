#![allow(unused)]

use library::storage::StorageBucket;
use library::types::Value;
use library::workflow::runtime::const_provider::RuntimeConstantProvider;
use library::workflow::runtime::source::{MutableSource, Source, SourceProvider};
use library::workflow::template::SourceDataVariant;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};

pub struct SourceMock {
    pub tpls: Vec<(String, Value)>,
}

impl SourceProvider for SourceMock {
    fn tpl(&self, key: &str) -> Option<&Value> {
        self.tpls.iter().find(|el| el.0 == key).map(|el| &el.1)
    }

    fn tpl_settings(&self, key: &str) -> Option<&Value> {
        todo!()
    }

    fn storage(&self, key: &str) -> Option<Value> {
        todo!()
    }

    fn global_storage(&self, key: &str) -> Option<Value> {
        todo!()
    }

    fn dao_const(&self, key: u8) -> Option<Value> {
        todo!()
    }

    fn props_action(&self, key: &str) -> Option<&Value> {
        todo!()
    }

    fn props_activity(&self, key: &str) -> Option<&Value> {
        todo!()
    }

    fn props_global(&self, key: &str) -> Option<&Value> {
        todo!()
    }

    fn storage_mut(&mut self) -> Option<&mut StorageBucket> {
        todo!()
    }

    fn global_storage_mut(&mut self) -> Option<&mut StorageBucket> {
        todo!()
    }
}

impl MutableSource for SourceMock {
    fn replace_storage(&mut self, new: StorageBucket) -> Option<StorageBucket> {
        todo!()
    }

    fn set_prop_action(&mut self, new: SourceDataVariant) -> Option<SourceDataVariant> {
        todo!()
    }

    fn take_storage(&mut self) -> Option<StorageBucket> {
        todo!()
    }

    fn take_global_storage(&mut self) -> Option<StorageBucket> {
        todo!()
    }

    fn set_prop_shared(&mut self, new: SourceDataVariant) -> Option<SourceDataVariant> {
        todo!()
    }

    fn unset_prop_action(&mut self) -> Option<SourceDataVariant> {
        todo!()
    }

    fn replace_global_storage(&mut self, new: StorageBucket) -> Option<StorageBucket> {
        todo!()
    }
}

impl Source for SourceMock {}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct PercentInput {
    pub total_possible: U128,
    pub actual: U128,
    pub decimals: u8,
}

impl PercentInput {
    pub fn new(total_possible: U128, actual: U128, decimals: u8) -> Self {
        Self {
            total_possible,
            actual,
            decimals,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct PercentResult {
    pub total_possible: U128,
    pub actual: U128,
    pub decimals: u8,
    pub result: u8,
}

impl PercentResult {
    pub fn from_input(input: &PercentInput, expected_result_u8: u8) -> Self {
        Self {
            total_possible: input.total_possible,
            actual: input.actual,
            decimals: input.decimals,
            result: expected_result_u8,
        }
    }
}
