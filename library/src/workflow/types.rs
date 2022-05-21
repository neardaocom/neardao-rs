use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
    AccountId,
};

use crate::{
    interpreter::{condition::Condition, expression::EExpr},
    types::datatype::{Datatype, Value},
};

use super::expression::Expression;

#[derive(
    BorshDeserialize, BorshSerialize, Deserialize, Serialize, Copy, Clone, Debug, PartialEq,
)]
//#[cfg_attr(not(target_arch = "wasm32"), derive())]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum DaoActionIdent {
    GroupAdd,
    GroupRemove,
    GroupUpdate,
    GroupAddMembers,
    GroupRemoveMembers,
    GroupRemoveRoles,
    GroupRemoveMemberRoles,
    SettingsUpdate,
    TagAdd,
    TagEdit,
    TagRemove,
    FtDistribute,
    WorkflowAdd,
    TreasuryAddPartition,
    RewardAdd,
    Event,
}

// TODO: Remove Debug in production.
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, PartialEq))]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum VoteScenario {
    Democratic,
    TokenWeighted,
}

// TODO: Remove Debug in production.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq))]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum ActivityRight {
    /// Anyone has the right.
    Anyone,
    /// Only group members.
    Group(u16),
    /// Only member in the group.
    GroupMember(u16, AccountId),
    /// Defined account.
    Account(AccountId),
    /// Any account_id with > 0 tokens.
    TokenHolder,
    /// Member in any group.
    Member,
    /// Members in the group with the role id.
    GroupRole(u16, u16),
    /// Only the group leader.
    GroupLeader(u16),
}

// TODO: Refactor.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug))]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
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

// TODO: Remove Debug in production.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq))]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
/// Defines source of value.
pub enum ArgSrc {
    /// User's input key name.
    User(String),
    /// Bind from template.
    ConstsTpl(String),
    /// Bind from template settings.
    ConstsSettings(String),
    ConstPropSettings(String),
    /// Bind from proposal settings.
    ConstActivityShared(String),
    /// Bind from proposal settings.
    ConstAction(String),
    Storage(String),
    GlobalStorage(String),
    /// Dao specific value known at runtime, eg. 0 means dao's account name.
    Const(u8),
}

// TODO: Remove Debug in production.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq))]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum ValueSrc {
    /// Source for value.
    Src(ArgSrc),
    /// Expression source which evaluates to the value.
    Expr(Expression),
    Value(Value),
}

// TODO: Remove Debug in production.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct BindDefinition {
    /// Key being binded.
    pub key: String,
    /// Value source for `key`.
    pub key_src: ValueSrc,
    /// Data related to collection object.
    pub collection_data: Option<CollectionBindData>,
}

// TODO: Remove Debug in production.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct CollectionBindData {
    /// Prefixes for nested collection objects.
    /// Defined as `Vec<String>` for forward-compatible changes.
    pub prefixes: Vec<String>,
    /// Defines type of binding for involved collection object.
    pub collection_binding_type: CollectionBindingStyle,
}

// TODO: Remove Debug in production.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq))]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum CollectionBindingStyle {
    /// Overwrites only user provided collection attributes.
    Overwrite,
    /// Makes up to defined number collection attributes.
    ForceSame(u8),
}
/// Object metadata.
/// Nested objects are referenced by specific `Datatype` in previous metadata.
/// These objects must be containerized, eg. in Vec.
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq, Clone))]
#[serde(crate = "near_sdk::serde")]
pub struct ObjectMetadata {
    pub arg_names: Vec<String>,
    pub arg_types: Vec<Datatype>,
}

// TODO: Remove Debug in production.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq))]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum Instruction {
    DeleteKey(String),
    DeleteKeyGlobal(String),
    /// User/Source provided value.
    StoreDynValue(String, ValueSrc),
    StoreValue(String, Value),
    StoreValueGlobal(String, Value),
    StoreFnCallResult(String, FnCallResultType),
    StoreFnCallResultGlobal(String, FnCallResultType),
    StoreWorkflow,
    /// Stores expression
    /// `FnCallResultType` arg defines if FnCallResult is required and what to deserialize it to.
    /// FnCall result will always be as last arg in values.
    StoreExpression(String, Vec<ValueSrc>, EExpr, Option<FnCallResultType>),
    StoreExpressionGlobal(String, Vec<ValueSrc>, EExpr, Option<FnCallResultType>),
    StoreExpressionBinded(String, Vec<Value>, EExpr, Option<FnCallResultType>),
    StoreExpressionGlobalBinded(String, Vec<Value>, EExpr, Option<FnCallResultType>),
    /// Conditional Jump.
    /// `FnCallResultType` arg defines if FnCallResult is required and what to deserialize it to.
    /// FnCall result will always be as last arg in values.
    Cond(Vec<ValueSrc>, Condition, Option<FnCallResultType>),
    CondBinded(Vec<Value>, Condition, Option<FnCallResultType>),
    Jump(u8),
    None,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq))]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum FnCallResultType {
    Datatype(Datatype),
}

impl FnCallResultType {
    pub fn into_datatype_ref(&self) -> Option<&Datatype> {
        match self {
            FnCallResultType::Datatype(d) => Some(d),
            _ => None,
        }
    }
}
