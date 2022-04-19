use std::{unimplemented, vec::Vec};

use library::types::activity_input::ValueCollection;
use library::workflow::types::FnCallMetadata;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::near_bindgen;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Default)]
pub struct Contract {
    fncall_metadata: Vec<FnCallMetadata>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(fncall_metadata: Vec<FnCallMetadata>) -> Self {
        Self { fncall_metadata }
    }

    pub fn bench_workflow(&self, _input: ValueCollection) {
        unimplemented!("Implement benchmark for new workflow inputs")
    }
}
