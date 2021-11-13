use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::AccountId;

use crate::CID;
use crate::file::{FileType, VFileMetadata};

//TODO: move from action.rs
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug,PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum TokenGroup {
    Council,
    Foundation,
    Community,
    Public,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum MemberGroup {
    Council,
    Foundation,
    Community,
    Public,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum PaymentPeriod {
    Daily,
    Weekly,
    Monthly
}

impl PaymentPeriod {
    pub fn to_nanos(&self) -> u64 {
        match self {
            PaymentPeriod::Daily => 86_400 * 10u64.pow(9),
            PaymentPeriod::Weekly =>  7 * 86_400 * 10u64.pow(9),
            PaymentPeriod::Monthly => 4 * 7 * 86_400 * 10u64.pow(9),
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq, Serialize))]
#[serde(crate = "near_sdk::serde")]
pub enum TxInput {
    Pay {
        amount_near: U128,
        account_id: AccountId,
    },
    AddMember {
        account_id: AccountId,
        group: TokenGroup,
    },
    RemoveMember {
        account_id: AccountId,
        group: TokenGroup,
    },
    RegularPayment {
        account_id: AccountId,
        amount_near: U128,
        since: u64,
        until: u64,
        period: PaymentPeriod,
    },
    GeneralProposal {
        title: String,
    },
    AddDocFile {
        cid: CID,
        metadata: VFileMetadata,
        new_tags: Vec<String>,
        new_category: Option<String>
    },
    InvalidateFile {
        cid: CID,
    },
    DistributeFT {
        total_amount: u32,
        from_group: TokenGroup,
        accounts: Vec<AccountId>,
    }
}


#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct ActionTx {
    pub actions: Vec<Action>,
}


#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)] //TODO Remove debug in production
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum ActionExecutionError {
    NotEnoughNears,
    NotEnoughFT,
    InvalidTimeInputs,
    CIDExists,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum Action {
    SendNear {
        account_id: AccountId,
        amount_near: u128,
    },
    AddMember {
        account_id: AccountId,
        group: TokenGroup,
    },
    RemoveMember {
        account_id: AccountId,
        group: TokenGroup,
    },
    RegularPayment {
        account_id: AccountId,
        amount_near: u128,
        since: u64,
        until: u64,
        period: PaymentPeriod,
    },
    GeneralProposal {
        title: String,
    },
    //TODO split into two actions ?
    AddFile {
        cid: CID,
        ftype: FileType,
        metadata: VFileMetadata,
        new_category: Option<String>,
        new_tags: Vec<String>
    },
    InvalidateFile {
        cid: CID,
    },
    DistributeFT {
        amount: u32,
        from_group: TokenGroup,
        accounts: Vec<AccountId>,
    }
}