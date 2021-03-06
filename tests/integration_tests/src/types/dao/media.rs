use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum ResourceType {
    Text(String),
    Link(String),
    Cid(CIDInfo),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct CIDInfo {
    pub ipfs: String,
    pub cid: String,
    pub mimetype: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct Media {
    pub proposal_id: Option<u32>,
    pub name: String,
    pub category: String,
    pub r#type: ResourceType,
    pub tags: Vec<u16>,
    pub version: String,
    pub valid: bool,
}
