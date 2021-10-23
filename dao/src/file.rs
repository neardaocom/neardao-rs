use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};

use crate::view::DocFileMetadata;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum VFileMetadata {
    //Prev (FileMetadata),
    Curr (FileMetadata),
}

impl VFileMetadata {
    pub fn migrate(self) -> Self {
        //TODO: implement when migrating
        self
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct FileMetadata 
{
    pub name: String,
    pub description: String,
    pub tags: Vec<u8>,
    pub category: u8,
    pub ext: String,
    pub v: String,
    pub valid: bool,
}

impl From<VFileMetadata> for FileMetadata {
    fn from(fm: VFileMetadata) -> Self {
        match fm {
            VFileMetadata::Curr(v) => v
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum FileType {
    Doc,
}
