use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
};

use crate::{types::activity_input::UserInput, FnCallId, MethodName};

use super::{
    expression::Expression,
    postprocessing::Postprocessing,
    types::{ArgSrc, DaoActionIdent, ValidatorRef},
};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct TemplateAction {
    //pub action: Terminality,
    pub exec_condition: Option<Expression>,
    pub input_validators: Vec<ValidatorRef>,
    pub action_data: ActionData,
    pub postprocessing: Option<Postprocessing>,
    pub must_succeed: bool,
    pub optional: bool,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq, Clone))]
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

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq, Clone))]
#[serde(crate = "near_sdk::serde")]
pub struct DaoActionData {
    pub name: DaoActionIdent,
    /// Deposit needed from caller.
    pub required_deposit: Option<ArgSrc>,
    pub inputs_definitions: Vec<Vec<ArgSrc>>,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq, Clone))]
#[serde(crate = "near_sdk::serde")]
pub struct FnCallData {
    pub id: FnCallIdType,
    pub tgas: u16,
    /// Deposit given by DAO for fncall.
    pub deposit: Option<ArgSrc>,
    pub inputs_definitions: Vec<Vec<ArgSrc>>,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq, Clone))]
#[serde(crate = "near_sdk::serde")]
/// Defines type of function call and source of its receiver.
/// Variants with prefix "Standard" define function calls for standart implementation methods, eg. FT NEP-141.
/// Reason is that we do not have to store same metadata multiple times but only once.
pub enum FnCallIdType {
    Static(FnCallId),
    Dynamic(ArgSrc, MethodName),
    StandardStatic(FnCallId),
    StandardDynamic(ArgSrc, MethodName),
}

#[derive(Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, Serialize))]
#[serde(crate = "near_sdk::serde")]
pub struct ActionInput {
    pub action: DaoActionOrFnCall,
    pub values: UserInput,
}

#[derive(Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, Serialize))]
#[serde(crate = "near_sdk::serde")]
pub enum DaoActionOrFnCall {
    DaoAction(DaoActionIdent),
    FnCall(FnCallId),
}

impl PartialEq<ActionData> for DaoActionOrFnCall {
    fn eq(&self, other: &ActionData) -> bool {
        match (self, other) {
            (Self::DaoAction(l0), ActionData::Action(d)) => *l0 == d.name,
            (Self::DaoAction(_), ActionData::FnCall(_)) => false,
            (Self::FnCall(l0), ActionData::FnCall(f)) => match &f.id {
                FnCallIdType::Static(s) => *l0 == *s,
                FnCallIdType::Dynamic(_, m) => *l0.0.as_str() == *m,
                FnCallIdType::StandardStatic(s) => *l0 == *s,
                FnCallIdType::StandardDynamic(_, m) => *l0.0.as_str() == *m,
            },
            (Self::FnCall(_), ActionData::Action(_)) => false,
            _ => false,
        }
    }
}
