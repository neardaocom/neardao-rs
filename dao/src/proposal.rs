use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::AccountId;
use std::collections::HashMap;

pub const PROPOSAL_DESC_MAX_LENGTH: usize = 256;

#[derive(BorshSerialize, BorshDeserialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum VProposal {
    //Prev(ProposalOld)
    Curr(Proposal),
}

impl VProposal {
    /// Migration method
    pub fn migrate(self) -> Self {
        //TODO: implement when migrating
        self
    }
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
    Invalid, // Not enough voters/tokens when time expired or could not apply tx
    Spam,
    Rejected,
    Accepted, // Accepted and winning transaction executed
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum VoteResult {
    Ok,
    AlreadyVoted,
    InvalidVote,
    VoteEnded,
}

// ---------------- NEW ----------------

#[derive(BorshSerialize, BorshDeserialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Proposal {
    pub created: u64,
    pub votes: HashMap<AccountId, u8>,
    pub state: ProposalState,
    pub workflow_id: u16,
    pub workflow_settings_id: u8,
    pub workflow_add_settings_id: bool,
}

impl Proposal {
    #[inline]
    pub fn new(created: u64, workflow_id: u16, workflow_settings_id: u8, workflow_add_settings_id: bool) -> Self {
        Proposal {
            created,
            votes: HashMap::new(),
            state: ProposalState::InProgress,
            workflow_id,
            workflow_settings_id,
            workflow_add_settings_id,
        }
    }
}
