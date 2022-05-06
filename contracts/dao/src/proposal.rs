use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::AccountId;
use std::collections::HashMap;

use crate::media::Media;
use crate::{ResourceId, StorageKey, TimestampSec};

pub const PROPOSAL_DESC_MAX_LENGTH: usize = 256;

#[derive(BorshSerialize, BorshDeserialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum VProposal {
    Curr(Proposal),
}

impl From<VProposal> for Proposal {
    fn from(fm: VProposal) -> Self {
        match fm {
            VProposal::Curr(p) => p,
            _ => unimplemented!(),
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum ProposalState {
    InProgress,
    /// Below quorum limit.
    Invalid,
    /// Above spam threshold.
    Spam,
    /// Below approve threshold.
    Rejected,
    Accepted,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum VoteResult {
    Ok,
    AlreadyVoted,
    NoRights,
    InvalidVote,
    VoteEnded,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Proposal {
    pub desc: ResourceId,
    pub created: TimestampSec,
    pub created_by: AccountId,
    pub votes: HashMap<AccountId, u8>,
    pub state: ProposalState,
    pub workflow_id: u16,
    pub workflow_settings_id: u8,
}

impl Proposal {
    #[inline]
    pub fn new(
        desc: ResourceId,
        created: u64,
        created_by: AccountId,
        workflow_id: u16,
        workflow_settings_id: u8,
    ) -> Self {
        Proposal {
            desc,
            created,
            created_by,
            votes: HashMap::new(),
            state: ProposalState::InProgress,
            workflow_id,
            workflow_settings_id,
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum ProposalContent {
    Media(Media),
}
