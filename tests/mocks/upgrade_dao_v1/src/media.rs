use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};

use crate::core::Contract;
use crate::ProposalId;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum ResourceType {
    Text(String),
    Link(String),
    CID(CIDInfo),
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct CIDInfo {
    pub ipfs: String,
    pub cid: String,
    pub mimetype: String,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Media {
    pub proposal_id: Option<ProposalId>,
    pub name: String,
    pub category: String,
    pub r#type: ResourceType,
    pub tags: Vec<usize>,
    pub version: String,
    pub valid: bool,
}

impl Contract {
    pub fn add_media(&mut self, media: &Media) -> u32 {
        self.media_last_id += 1;
        self.media.insert(&self.media_last_id, media);
        self.media_last_id
    }
}
