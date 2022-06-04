use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};

use crate::contract::Contract;
use crate::ProposalId;

// TODO: Remove all Debug in production.

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq))]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum ResourceType {
    Text(String),
    Link(String),
    Cid(CIDInfo),
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct CIDInfo {
    pub ipfs: String,
    pub cid: String,
    pub mimetype: String,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Media {
    pub proposal_id: Option<ProposalId>,
    pub name: String,
    pub category: String,
    pub r#type: ResourceType,
    pub tags: Vec<u16>,
    pub version: String,
    pub valid: bool,
}

impl Contract {
    pub fn media_add(&mut self, media: &Media) -> u32 {
        self.media_last_id += 1;
        self.media.insert(&self.media_last_id, media);
        self.media_last_id
    }
    pub fn media_update(&mut self, id: u32, media: &Media) {
        if let Some(_) = self.media.get(&id) {
            self.media.insert(&id, media);
        }
    }
}
