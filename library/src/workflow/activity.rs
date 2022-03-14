use std::marker::PhantomData;

use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    json_types::{U128, U64},
    serde::{Deserialize, Serialize},
    serde_json,
};

use crate::{
    interpreter::{condition::Condition, error::EvalError},
    storage::StorageBucket,
    types::{DataType, DataTypeDef},
    ActivityId, FnCallId, MethodName,
};

use super::{
    expression::{CondOrExpr, Expression},
    types::{ArgSrc, DaoActionIdent, ValidatorRef, ValueContainer},
};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
/// Wrapper around `TemplateActivity`.
pub enum Activity {
    Init,
    Activity(TemplateActivity),
}

impl Activity {
    pub fn into_activity(self) -> Option<TemplateActivity> {
        match self {
            Activity::Init => None,
            Activity::Activity(a) => Some(a),
        }
    }

    pub fn activity_as_ref(&self) -> Option<&TemplateActivity> {
        match self {
            &Activity::Init => None,
            &Activity::Activity(ref a) => Some(a),
        }
    }
}

/// Defines activity relation to workflow finish.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum Terminality {
    NonTerminal = 0,
    /// Can be closed by user.
    User = 1,
    /// Can be closed by anyone.
    Automatic = 2,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct TemplateActivity {
    pub code: String,
    pub actions: Vec<TemplateAction>,
    /// Execution can be done by anyone anytime.
    pub automatic: bool,
    /// Workflow can be autoclosed when this was successfull.
    pub terminal: Terminality,
    pub postprocessing: Option<Postprocessing>,
}

impl TemplateActivity {
    pub fn get_dao_action_type(&self, id: u8) -> Option<DaoActionIdent> {
        match self.actions.get(id as usize) {
            Some(action) => match &action.action_data {
                ActionData::FnCall(_) => None,
                ActionData::Action(a) => Some(a.name.clone()),
            },
            None => None,
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct TemplateAction {
    pub action: Terminality,
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
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq, Clone))]
#[serde(crate = "near_sdk::serde")]
pub struct DaoActionData {
    pub name: DaoActionIdent,
    pub deposit: Option<ArgSrc>,
    pub inputs_definitions: Vec<Vec<ArgSrc>>,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq, Clone))]
#[serde(crate = "near_sdk::serde")]
pub struct FnCallData {
    pub id: FnCallIdType,
    pub tgas: u16,
    pub deposit: U128,
    pub inputs_definitions: Vec<Vec<ArgSrc>>,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq, Clone))]
#[serde(crate = "near_sdk::serde")]
pub enum FnCallIdType {
    Static(FnCallId),
    Dynamic(ArgSrc, MethodName),
}

//TODO move all to instructions
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum Instruction {
    DeleteKey(String),
    StoreDynValue(String, ArgSrc),
    StoreValue(String, DataType),
    StoreFnCallResult(String, DataTypeDef),
    StoreExpression(String, Expression),
    Cond(Condition),
}
/// Simple post-fncall instructions which say what to do based on FnCall result.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Postprocessing {
    pub storage_key: String,
    pub instructions: Vec<Instruction>,
}

impl Postprocessing {
    pub fn bind_and_convert(mut self, value_source: ValueContainer<&[DataType]>) -> Self {
        for ins in self.instructions.iter_mut() {}

        Postprocessing {
            storage_key: self.storage_key,
            instructions: self.instructions,
        }
    }

    pub fn postprocess(
        self,
        fn_result_val: Vec<u8>,
        inner_value: Option<DataType>,
        storage: &mut StorageBucket,
    ) -> bool {
        /*         match self.op_type {
            Instruction::DeleteActionValue(key) => {
                storage.remove_data(&key);
            }
            Instruction::SaveBind(_) => Some(inner_value.unwrap()),
            Instruction::SaveUserValue(_) => Some(inner_value.unwrap()),
            Instruction::StoreFnCallResult(t) => {
                let result = match t {
                    DataTypeDef::String(_) => {
                        DataType::String(serde_json::from_slice::<String>(&fn_result_val).unwrap())
                    }
                    DataTypeDef::Bool(_) => {
                        DataType::Bool(serde_json::from_slice::<bool>(&fn_result_val).unwrap())
                    }
                    DataTypeDef::U8(_) => {
                        DataType::U8(serde_json::from_slice::<u8>(&fn_result_val).unwrap())
                    }
                    DataTypeDef::U16(_) => {
                        DataType::U16(serde_json::from_slice::<u16>(&fn_result_val).unwrap())
                    }
                    DataTypeDef::U32(_) => {
                        DataType::U32(serde_json::from_slice::<u32>(&fn_result_val).unwrap())
                    }
                    DataTypeDef::U64(_) => {
                        DataType::U64(serde_json::from_slice::<U64>(&fn_result_val).unwrap())
                    }
                    DataTypeDef::U128(_) => {
                        DataType::U128(serde_json::from_slice::<U128>(&fn_result_val).unwrap())
                    }
                    DataTypeDef::VecString => DataType::VecString(
                        serde_json::from_slice::<Vec<String>>(&fn_result_val).unwrap(),
                    ),
                    DataTypeDef::VecU8 => {
                        DataType::VecU8(serde_json::from_slice::<Vec<u8>>(&fn_result_val).unwrap())
                    }
                    DataTypeDef::VecU16 => DataType::VecU16(
                        serde_json::from_slice::<Vec<u16>>(&fn_result_val).unwrap(),
                    ),
                    DataTypeDef::VecU32 => DataType::VecU32(
                        serde_json::from_slice::<Vec<u32>>(&fn_result_val).unwrap(),
                    ),
                    DataTypeDef::VecU64 => DataType::VecU64(
                        serde_json::from_slice::<Vec<U64>>(&fn_result_val).unwrap(),
                    ),
                    DataTypeDef::VecU128 => DataType::VecU128(
                        serde_json::from_slice::<Vec<U128>>(&fn_result_val).unwrap(),
                    ),
                    _ => {
                        unimplemented!();
                    }
                };
                storage.add_data(&self.storage_key, &result);
            }
            Instruction::SaveValue(val) => Some(val),
        } */
        true
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Transition {
    pub activity_id: ActivityId,
    pub cond: Option<Expression>,
    pub time_from_cond: Option<Expression>,
    pub time_to_cond: Option<Expression>,
}
