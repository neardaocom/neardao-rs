use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};

#[derive(BorshDeserialize, BorshSerialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
pub enum VConfig {
    //Prev(ConfigOld),
    Curr(Config),
}

impl VConfig {
    pub fn migrate(self) -> Self {
        //TODO: implement on migration
        self
    }
}

/// User provided config type
#[derive(Deserialize)]
#[cfg_attr(
    not(target_arch = "wasm32"),
    derive(Clone, Debug, PartialEq, Serialize)
)]
#[serde(crate = "near_sdk::serde")]
pub struct ConfigInput {
    pub name: String,
    pub lang: String,
    pub slogan: String,
    pub council_share: Option<u8>,
    pub foundation_share: Option<u8>,
    pub community_share: Option<u8>,
    pub description: Option<String>,
    pub vote_spam_threshold: Option<u8>, //how many percent of total tokens voters needed for proposal to go to spam, relative to voted token weight amount
}

#[derive(BorshDeserialize, BorshSerialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
pub struct Config {
    pub name: String,
    pub lang: String,
    pub slogan: String,
    pub council_share: u8,
    pub foundation_share: Option<u8>,
    pub community_share: Option<u8>,
    pub description: String,
    pub vote_spam_threshold: u8, //how many percent of total tokens voters needed for proposal to go to spam, relative to voted token weight amount
}

impl From<VConfig> for Config {
    fn from(config: VConfig) -> Self {
        match config {
            VConfig::Curr(c) => c,
            _ => unreachable!(),
        }
    }
}

impl From<ConfigInput> for VConfig {
    fn from(input: ConfigInput) -> Self {
        VConfig::Curr(Config {
            name: input.name,
            lang: input.lang,
            slogan: input.slogan,
            council_share: input.council_share.unwrap(),
            foundation_share: input.foundation_share,
            community_share: input.community_share,
            description: input.description.unwrap(),
            vote_spam_threshold: input.vote_spam_threshold.unwrap(),
        })
    }
}
