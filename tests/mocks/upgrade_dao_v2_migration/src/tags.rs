//! Tags for DAO
//! Simple storage for strings which are then referenced by integer key.
//! Last inserted key is kept to avoid assigning same key to two different tags during the tags lifetime in DAO.
//! Contains no logic exepct basic CRUD methods.

use std::collections::{hash_map::Iter, HashMap};

use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
};

use crate::{core::Contract, TagId};

#[derive(Deserialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct TagInput {
    pub category: String,
    pub values: Vec<String>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Tags {
    last_id: u16,
    map: HashMap<TagId, String>,
}
