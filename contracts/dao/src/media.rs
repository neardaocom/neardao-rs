//TODO move to separate smartcontract

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};

use crate::ProposalId;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
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
    pub proposal_id: ProposalId,
    pub name: String,
    pub category: String,
    pub media_type: ResourceType,
    pub tags: Vec<usize>,
    pub version: String,
    pub valid: bool,
}
