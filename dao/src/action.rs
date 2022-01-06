use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{U128, WrappedDuration, WrappedTimestamp};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::AccountId;

use crate::CID;
use crate::file::{FileType, VFileMetadata};

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug,PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum TokenGroup {
    Council,
    Public,
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
    },
    RightForActionCall {
        to: RightTarget,
        rights: Vec<ActionGroupRight>,
        time_from: Option<WrappedTimestamp>,
        time_to: Option<WrappedTimestamp>
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
pub enum TxValidationErr {
    NotEnoughGas,
    NotEnoughNears,
    NotEnoughFT,
    InvalidTimeInputs,
    CIDExists,
    GroupForbidden,
    UserAlreadyInGroup,
    UserNotInGroup,
    Custom(String)
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
    },
    AddRightsForActionGroup {
        to: RightTarget,
        rights: Vec<ActionGroupRight>,
        time_from: u64,
        time_to: u64,
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum RightTarget {
    Group {
        value: TokenGroup   
    },
    Users {
        values: Vec<AccountId>
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum ActionGroupRight {
    RefFinance,
    SkywardFinance
}

/// ActionGroup structure represents input type for external action calls from privileged user
/// One ActionGroup will be splitted into 1..N actions
#[derive(Deserialize, Clone, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, Serialize))]
#[serde(crate = "near_sdk::serde")]
pub enum ActionGroupInput {
    RefRegisterTokens,
    RefAddPool { fee: Option<u32> },
    RefAddLiquidity { pool_id: u32, amount_near: U128, amount_ft: U128 },
    RefWithdrawLiquidity { pool_id: u32, shares: U128, min_ft: Option<U128>, min_near: Option<U128> },
    RefWithdrawDeposit { token_id: AccountId, amount: U128 },
    SkyCreateSale { title: String, url: String, amount_ft: U128, out_token_id: AccountId, time_from: WrappedTimestamp, duration: WrappedDuration },
}