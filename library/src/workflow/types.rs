use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
    AccountId,
};

use crate::{
    storage::StorageBucket,
    types::{DataType, DataTypeDef},
    BindId, Consts, ExpressionId, ObjectId, ValidatorId,
};

pub struct ValueContainer<'a, T: AsRef<[DataType]>> {
    pub dao_consts: &'a Consts,
    pub tpl_consts: &'a T,
    pub settings_consts: &'a T,
    pub proposal_consts: &'a T,
    pub storage: &'a StorageBucket,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum DaoActionIdent {
    GroupAdd,
    GroupRemove,
    GroupUpdate,
    GroupAddMembers,
    GroupRemoveMember,
    SettingsUpdate,
    MediaAdd,
    MediaInvalidate,
    MediaRemove,
    TagAdd,
    TagEdit,
    TagRemove,
    FtDistribute,
    TreasurySendFt,
    TreasurySendFtContract,
    TreasurySendNft,
    TreasurySendNFtContract,
    TreasurySendNear,
    WorkflowAdd,
    Event,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum VoteScenario {
    Democratic,
    TokenWeighted,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)] //Remove clone + debug in prod
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum ActivityRight {
    Anyone,
    Group(u16),
    GroupMember(u16, String),
    Account(AccountId),
    TokenHolder,
    Member,
    GroupRole(u16, u16),
    GroupLeader(u16),
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub struct ValidatorRef {
    pub v_type: ValidatorType,
    pub obj_id: ObjectId,
    pub val_id: ValidatorId,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum ValidatorType {
    Simple,
    Collection,
}

impl ValidatorRef {
    pub fn is_simple(&self) -> bool {
        self.v_type == ValidatorType::Simple
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum ActionResult {
    Ok,
    Finished,
    NoRights,
    NotEnoughDeposit,
    TransitionNotPossible,
    ProposalNotAccepted,
    Postprocessing,
    MaxTransitionLimitReached,
    TransitionCondFailed,
    ActivityCondFailed,
    ErrValidation,
    ErrPostprocessing,
    ErrTimeLimit,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
/// Defines source of value.
pub enum ArgSrc {
    /// User's input
    User(u8),
    /// Bind from template.
    ConstsTpl(BindId),
    ConstsSettings(BindId),
    /// Bind from proposal settings.
    ConstProps(BindId),
    Storage(String),
    Expression(ExpressionId),
    Object(ObjectId),
    VecObject(ObjectId),
    /// Dao specific value known at runtime, eg. 0 means dao's account name.
    Const(u8),
}

// Represents object schema
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq, Clone))]
#[serde(crate = "near_sdk::serde")]
pub struct FnCallMetadata {
    pub arg_names: Vec<String>,
    pub arg_types: Vec<DataTypeDef>,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum ActionType {
    DaoAction,
    FnCall,
}
