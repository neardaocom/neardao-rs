use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
    AccountId,
};

use crate::{
    interpreter::{condition::Condition, expression::EExpr},
    storage::StorageBucket,
    types::datatype::{Datatype, Value},
    Consts, ObjectId, ValidatorId,
};

use super::expression::Expression;

// TODO: replace with Source trait
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
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq, Debug))]
#[serde(crate = "near_sdk::serde")]
/// Defines source of value.
pub enum ArgSrc {
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
    // Special case where expression result is used as source value.
    //Expression(ExpressionId),
    /// Dao specific value known at runtime, eg. 0 means dao's account name.
    Const(u8),
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq, Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum SrcOrExpr {
    /// Source for value.
    Src(ArgSrc),
    /// Expression sources which evaluates to the value.
    Expr(Expression),
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct BindDefinition {
    /// Key being binded.
    pub key: String,
    /// Value source for `key`.
    pub key_src: SrcOrExpr,
    /// Prefixes for nested collection objects.
    /// Defined as Vec<String> for forward-compatible changes.
    pub prefixes: Vec<String>,
    pub is_collection: bool,
    //pub expression: Option<Expression>,
}

// Represents object schema
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq, Clone))]
#[serde(crate = "near_sdk::serde")]
pub struct FnCallMetadata {
    pub arg_names: Vec<String>,
    pub arg_types: Vec<Datatype>,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum ActionType {
    DaoAction,
    FnCall,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum Instruction {
    DeleteKey(String),
    DeleteKeyGlobal(String),
    /// User/Source provided value.
    StoreDynValue(String, ArgSrc),
    StoreValue(String, Value),
    StoreValueGlobal(String, Value),
    StoreFnCallResult(String, Datatype),
    StoreFnCallResultGlobal(String, Datatype),
    StoreWorkflow,
    /// Stores expression
    /// 3th param defines if FnCallResult is required and what to deserialize it to.
    /// FnCall result will always be as last arg in values.
    StoreExpression(String, Vec<ArgSrc>, EExpr, Option<Datatype>),
    StoreExpressionGlobal(String, Vec<ArgSrc>, EExpr, Option<Datatype>),
    StoreExpressionBinded(String, Vec<Value>, EExpr, Option<Datatype>),
    StoreExpressionGlobalBinded(String, Vec<Value>, EExpr, Option<Datatype>),
    /// Conditional Jump.
    /// 3th param defines if FnCallResult is required and what to deserialize it to.
    /// FnCall result will always be as last arg in values.
    Cond(Vec<ArgSrc>, Condition, Option<Datatype>),
    CondBinded(Vec<Value>, Condition, Option<Datatype>),
    Jump(u8),
    None,
}
