use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
    AccountId,
};

use crate::{types::activity_input::UserInput, MethodName};

use super::{
    expression::Expression,
    postprocessing::Postprocessing,
    types::{ArgSrc, BindDefinition, DaoActionIdent},
    validator::Validator,
};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct TemplateAction {
    pub exec_condition: Option<Expression>,
    pub validators: Vec<Validator>,
    pub action_data: ActionData,
    pub postprocessing: Option<Postprocessing>,
    pub must_succeed: bool,
    pub optional: bool,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq, Clone))]
#[serde(crate = "near_sdk::serde")]
pub enum ActionData {
    FnCall(FnCallData),
    Action(DaoActionData),
    None,
}

impl ActionData {
    pub fn try_into_action_data(self) -> Option<DaoActionData> {
        match self {
            Self::Action(data) => Some(data),
            _ => None,
        }
    }

    pub fn try_into_fncall_data(self) -> Option<FnCallData> {
        match self {
            Self::FnCall(data) => Some(data),
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

impl DaoActionData {
    pub fn is_event(&self) -> bool {
        self.name == DaoActionIdent::Event
    }
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
// TODO: Remove Debug and Clone in production.
#[derive(Deserialize, Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize))]
#[serde(crate = "near_sdk::serde")]
pub enum DaoActionOrFnCall {
    DaoAction(DaoActionIdent),
    FnCall(AccountId, MethodName),
}

impl PartialEq<ActionData> for DaoActionOrFnCall {
    fn eq(&self, other: &ActionData) -> bool {
        match (self, other) {
            (Self::DaoAction(l0), ActionData::Action(d)) => *l0 == d.name,
            (Self::DaoAction(_), ActionData::FnCall(_)) => false,
            (Self::FnCall(a0, m0), ActionData::FnCall(f)) => match &f.id {
                FnCallIdType::Static(a, m) => *a0 == *a && *m0 == *m,
                FnCallIdType::Dynamic(_, m) => *m0.as_str() == *m,
                FnCallIdType::StandardStatic(a, m) => *a0 == *a && *m0 == *m,
                FnCallIdType::StandardDynamic(_, m) => *m0.as_str() == *m,
            },
            (Self::FnCall(_, _), ActionData::Action(_)) => false,
            _ => false,
        }
    }
}
