use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::AccountId;
use std::collections::HashMap;
use std::ops::Add;

use crate::CID_MAX_LENGTH;
use crate::action::{ActionTx, TxInput};
use crate::vote_policy::VoteConfig;

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

#[derive(BorshSerialize, BorshDeserialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
/// User provided proposal type
pub struct ProposalInput {
    pub tags: Vec<String>,
    pub description: Option<String>,
    pub description_cid: Option<String>,
}

impl ProposalInput {
    /// Checks description's and cid's max lengths
    /// Panics if above limits
    pub(crate) fn assert_valid(&self) {
        if let Some(desc) = self.description.as_ref() {
            assert!(desc.len() > 0 && desc.len() <= PROPOSAL_DESC_MAX_LENGTH);
        }

        if let Some(cid) = self.description_cid.as_ref() {
            assert!(cid.len() > 0 && cid.len() <= CID_MAX_LENGTH.into());
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum ProposalKindIdent {
    Pay,
    AddMember,
    RemoveMember,
    RegularPayment,
    GeneralProposal,
    AddDocFile,
    InvalidateFile,
    DistributeFT,
}

impl ProposalKindIdent {
    pub fn get_ident_from(tx_input: &TxInput) -> Self {
        match tx_input {
            TxInput::Pay { .. } => ProposalKindIdent::Pay,
            TxInput::AddMember { .. } => ProposalKindIdent::AddMember,
            TxInput::RemoveMember { .. } => ProposalKindIdent::RemoveMember,
            TxInput::RegularPayment { .. } => ProposalKindIdent::RegularPayment,
            TxInput::GeneralProposal { .. } => ProposalKindIdent::GeneralProposal,
            TxInput::AddDocFile { .. } => ProposalKindIdent::AddDocFile,
            TxInput::InvalidateFile { .. } => ProposalKindIdent::InvalidateFile,
            TxInput::DistributeFT { .. } => ProposalKindIdent::DistributeFT,
            _ => unimplemented!(),
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Proposal {
    pub uuid: u32,
    pub proposed_by: AccountId,
    pub description: Option<String>,
    pub description_cid: Option<String>,
    pub tags: Vec<String>,
    pub status: ProposalStatus,
    pub votes: HashMap<AccountId, u8>, // account id, vote id
    pub transactions: ActionTx,        // id of transaction should match vote id above
    pub duration_to: u64,
    pub waiting_open_duration: u64,
    pub quorum: u8,
    pub approve_threshold: u8,
    pub vote_only_once: bool,
}

impl Proposal {}

impl Proposal {
    pub fn new(
        uuid: u32,
        proposer: AccountId,
        input: ProposalInput,
        tx: ActionTx,
        vote_policy: VoteConfig,
        current_time: u64,
    ) -> Self {
        Proposal {
            uuid: uuid,
            proposed_by: proposer,
            description: input.description,
            description_cid: input.description_cid,
            tags: input.tags,
            status: ProposalStatus::InProgress,
            votes: HashMap::new(),
            transactions: tx,
            duration_to: vote_policy.duration.add(current_time),
            waiting_open_duration: vote_policy.waiting_open_duration,
            quorum: vote_policy.quorum,
            approve_threshold: vote_policy.approve_threshold,
            vote_only_once: vote_policy.vote_only_once,
        }
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
pub enum ProposalStatus {
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
