use std::collections::HashMap;

use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
    AccountId,
};

use crate::{
    types::{activity_input::UserInput, datatype::Datatype},
    MethodName,
};

use super::{
    expression::Expression,
    postprocessing::Postprocessing,
    types::{ArgSrc, BindDefinition, DaoActionIdent},
    validator::Validator,
};

// TODO: Remove Debug in production.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct TemplateAction {
    pub exec_condition: Option<Expression>,
    pub validators: Vec<Validator>,
    pub action_data: ActionType,
    pub postprocessing: Option<Postprocessing>,
    pub must_succeed: bool,
    pub optional: bool,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq, Clone))]
#[serde(crate = "near_sdk::serde")]
pub enum ActionType {
    FnCall(FnCallData),
    Action(DaoActionData),
    Event(EventData),
    SendNear(ArgSrc, ArgSrc),
    None,
}

impl ActionType {
    pub fn try_into_action_data(self) -> Option<DaoActionData> {
        match self {
            Self::Action(data) => Some(data),
            _ => None,
        }
    }

    pub fn try_into_event_data(self) -> Option<EventData> {
        match self {
            Self::Event(data) => Some(data),
            _ => None,
        }
    }

    pub fn try_into_fncall_data(self) -> Option<FnCallData> {
        match self {
            Self::FnCall(data) => Some(data),
            _ => None,
        }
    }
    pub fn is_fncall(&self) -> bool {
        match self {
            Self::FnCall(_) => true,
            _ => false,
        }
    }

    pub fn try_into_send_near_sources(self) -> Option<(ArgSrc, ArgSrc)> {
        match self {
            Self::SendNear(a1, a2) => Some((a1, a2)),
            _ => None,
        }
    }
}

// TODO: Remove Debug and Clone in production.
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct DaoActionData {
    pub name: DaoActionIdent,
    /// Deposit needed from caller.
    /// Binded dynamically.
    pub required_deposit: Option<ArgSrc>,
    pub binds: Vec<BindDefinition>,
}
// TODO: Remove Debug and Clone in production.
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct EventData {
    /// Event code.
    pub code: String,
    /// Map of expected keys and values.
    pub expected_input: Option<Vec<(String, Datatype)>>,
    /// Binded dynamically.
    pub required_deposit: Option<ArgSrc>,
    pub binds: Vec<BindDefinition>,
}

// TODO: Remove Debug and Clone in production.
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq, Clone))]
#[serde(crate = "near_sdk::serde")]
pub struct FnCallData {
    pub id: FnCallIdType,
    pub tgas: u16,
    /// Deposit for function call given by executing contract.
    pub deposit: Option<ArgSrc>,
    pub binds: Vec<BindDefinition>,
}

// TODO: Remove Debug and Clone in production.
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq))]
#[serde(crate = "near_sdk::serde")]
/// Defines type of function call and source of its receiver.
/// Variants with prefix "Standard" define function calls for standart implementation methods, eg. FT NEP-141.
/// Reason is that we do not have to store same metadata multiple times but only once.
pub enum FnCallIdType {
    Static(AccountId, MethodName),
    Dynamic(ArgSrc, MethodName),
    StandardStatic(AccountId, MethodName),
    StandardDynamic(ArgSrc, MethodName),
}

// TODO: Remove Debug and Clone in production.
#[derive(Deserialize, Debug, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize))]
#[serde(crate = "near_sdk::serde")]
pub struct ActionInput {
    pub action: DaoActionOrFnCall,
    pub values: UserInput,
}

// TODO: Update structure.
// TODO: Remove Debug and Clone in production.
#[derive(Deserialize, Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize))]
#[serde(crate = "near_sdk::serde")]
pub enum DaoActionOrFnCall {
    DaoAction(DaoActionIdent),
    FnCall(AccountId, MethodName),
    SendNear,
}

impl PartialEq<ActionType> for DaoActionOrFnCall {
    fn eq(&self, other: &ActionType) -> bool {
        match (self, other) {
            (Self::DaoAction(l0), ActionType::Action(d)) => *l0 == d.name,
            (Self::DaoAction(_), ActionType::FnCall(_)) => false,
            (Self::FnCall(a0, m0), ActionType::FnCall(f)) => match &f.id {
                FnCallIdType::Static(a, m) => *a0 == *a && *m0 == *m,
                FnCallIdType::Dynamic(_, m) => *m0.as_str() == *m,
                FnCallIdType::StandardStatic(a, m) => *a0 == *a && *m0 == *m,
                FnCallIdType::StandardDynamic(_, m) => *m0.as_str() == *m,
            },
            (Self::FnCall(_, _), ActionType::Action(_)) => false,
            (Self::SendNear, ActionType::SendNear(_, _)) => true,
            _ => false,
        }
    }
}
