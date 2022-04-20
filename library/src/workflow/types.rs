use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
    AccountId,
};

use crate::{
    storage::StorageBucket,
    types::datatype::{Datatype, Value},
    BindId, Consts, ExpressionId, ObjectId, ValidatorId,
};

pub struct ValueContainer<'a, T: AsRef<[Value]>> {
    pub dao_consts: &'a Consts,
    pub tpl_consts: &'a T,
    pub settings_consts: &'a T,
    pub activity_shared_consts: Option<&'a T>,
    pub action_proposal_consts: Option<&'a T>,
    pub storage: Option<&'a mut StorageBucket>,
    pub global_storage: &'a mut StorageBucket,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Copy, Clone, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum DaoActionIdent {
    GroupAdd,
    GroupRemove,
    GroupUpdate,
    GroupAddMembers,
    GroupRemoveMember,
    SettingsUpdate,
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
    GroupMember(u16, AccountId),
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
pub enum ActivityResult {
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
    /// User's input - defines arg pos.
    User(u8),
    /// User's input - defines obj and arg pos.
    UserObj(u8, u8),
    /// Bind from template.
    ConstsTpl(BindId),
    ConstsSettings(BindId),
    /// Bind from proposal settings.
    ConstActivityShared(BindId),
    ConstAction(BindId),
    Storage(String),
    GlobalStorage(String),
    Expression(ExpressionId),
    Object(ObjectId),
    VecObject(ObjectId),
    /// Dao specific value known at runtime, eg. 0 means dao's account name.
    Const(u8),
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
/// Defines source of value.
pub enum ArgSrcNew {
    /// User's input - defines arg pos.
    User(String),
    /// Bind from template.
    ConstsTpl(String),
    ConstsSettings(String),
    /// Bind from proposal settings.
    ConstActivityShared(String),
    ConstAction(String),
    Storage(String),
    GlobalStorage(String),
    Expression(ExpressionId),
    /// Dao specific value known at runtime, eg. 0 means dao's account name.
    Const(u8),
}

// Represents object schema
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq, Clone))]
#[serde(crate = "near_sdk::serde")]
pub struct FnCallMetadata {
    pub arg_names: Vec<String>,
    pub arg_types: Vec<Datatype>,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq, Clone))]
#[serde(crate = "near_sdk::serde")]
pub struct FnCallMetadataNew {
    pub objects_count: u8,
    pub arg_names: Vec<String>,
    pub arg_type: Vec<Datatype>,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum ActionType {
    DaoAction,
    FnCall,
}
