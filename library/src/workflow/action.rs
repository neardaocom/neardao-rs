use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
    AccountId,
};

use crate::{types::Datatype, MethodName};

use super::{
    postprocessing::Postprocessing,
    runtime::activity_input::UserInput,
    types::{BindDefinition, DaoActionIdent, ValueSrc},
    validator::Validator,
};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, PartialEq))]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum InputSource {
    /// User inputs.
    User,
    /// Action constants.
    PropSettings,
}

// TODO: Remove Debug in production.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct TemplateAction {
    pub exec_condition: Option<ValueSrc>,
    pub validators: Vec<Validator>,
    pub action_data: ActionData,
    pub input_source: InputSource,
    pub postprocessing: Option<Postprocessing>,
    pub optional: bool,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq, Clone))]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum ActionData {
    FnCall(FnCallData),
    Action(DaoActionData),
    SendNear(ValueSrc, ValueSrc),
    Stake(ValueSrc, ValueSrc),
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
    pub fn is_fncall(&self) -> bool {
        matches!(self, Self::FnCall(_))
    }

    pub fn try_into_send_near_sources(self) -> Option<(ValueSrc, ValueSrc)> {
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
    /// Code in case of `DaoActionIdent::Event`.
    pub code: Option<String>,
    /// Map of expected keys and values in case of `DaoActionIdent::Event`.
    pub expected_input: Option<Vec<(String, Datatype)>>,
    /// Deposit needed from caller.
    /// Binded dynamically.
    pub required_deposit: Option<ValueSrc>,
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
    pub deposit: Option<ValueSrc>,
    pub binds: Vec<BindDefinition>,
    pub must_succeed: bool,
}

// TODO: Remove Debug and Clone in production.
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq))]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
/// Defines type of function call and source of its receiver.
/// Variants with prefix "Standard" define function calls for standart implementation methods, eg. FT NEP-141.
/// Reason is that we do not have to store same metadata multiple times but only once.
pub enum FnCallIdType {
    /// Receiver account_id is defined by workflow.
    Static(AccountId, MethodName),
    /// Receiver account_id is defined by dynamically.
    Dynamic(ValueSrc, MethodName),
    /// Standard call version of `Static`.
    StandardStatic(AccountId, MethodName),
    /// Standard call version of `Dynamic`.
    StandardDynamic(ValueSrc, MethodName),
}

// TODO: Remove Debug and Clone in production.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ActionInput {
    pub action: ActionInputType,
    pub values: UserInput,
}

// TODO: Update structure.
// TODO: Remove Debug and Clone in production.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum ActionInputType {
    DaoAction(DaoActionIdent),
    FnCall(AccountId, MethodName),
    Event(String),
    SendNear,
    Stake,
}

impl PartialEq<ActionData> for ActionInputType {
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
            (Self::SendNear, ActionData::SendNear(_, _)) => true,
            (Self::Event(code), ActionData::Action(d)) => {
                d.name == DaoActionIdent::Event && code == d.code.as_ref().unwrap_or(&"".into())
            }
            (Self::Stake, ActionData::Stake(_, _)) => true,
            _ => unimplemented!(),
        }
    }
}
