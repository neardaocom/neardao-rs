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
    TagUpdate,
    TagRemove,
    FtDistribute,
    WorkflowAdd,
    TreasuryAddPartition,
    PartitionAddNear,
    RewardAdd,
    RewardUpdate,
    Event,
    MediaAdd,
    MediaUpdate,
    MediaRemove,
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
    /// Members in the group with the role id.
    GroupRole(u16, u16),
    /// Only the group leader.
    GroupLeader(u16),
    /// Defined account.
    Account(AccountId),
    /// Any account with > 0 staked vote tokens in the DAO.
    TokenHolder,
    /// Member in any group.
    Member,
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
pub enum Src {
    /// User's input key name.
    User(String),
    /// Bind from template.
    Tpl(String),
    /// Bind from template settings.
    TplSettings(String),
    /// Bind from proposal settings - constants.
    PropSettings(String),
    /// Bind from proposal settings.
    Activity(String),
    /// Bind from proposal settings.
    Action(String),
    Storage(String),
    GlobalStorage(String),
    /// Specific value known at runtime.
    /// Eg. 0 means dao's account name in case of DAO contract.
    Runtime(u8),
}

impl Src {
    pub fn with_new_user_key(&self, key: String) -> Result<Self, &'static str> {
        match self {
            Src::User(_) => Ok(Src::User(key)),
            _ => Err("Invalid variant of self."),
        }
    }
}

// TODO: Remove Debug in production.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq))]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum ValueSrc {
    /// Source for value.
    Src(Src),
    /// Expression source which evaluates to the value.
    Expr(Expression),
    /// Constant value.
    Value(Value),
}

impl ValueSrc {
    pub fn is_user_input(&self) -> bool {
        match self {
            ValueSrc::Src(src) => match src {
                Src::User(_) => true,
                _ => false,
            },
            _ => false,
        }
    }
}

// TODO: Remove Debug in production.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct BindDefinition {
    /// Key being binded.
    pub key: String,
    /// Value source for `key`.
    pub value: ValueSrc,
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
        }
    }
}
