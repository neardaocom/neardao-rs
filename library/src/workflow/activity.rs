use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    json_types::{U128, U64},
    serde::{Deserialize, Serialize},
    serde_json,
};

use crate::{
    functions::get_value_from_source,
    interpreter::condition::Condition,
    storage::StorageBucket,
    types::{DataType, DataTypeDef},
    ActivityId, FnCallId, MethodName, ObjectValues,
};

use super::{
    expression::Expression,
    template::Template,
    types::{ArgSrc, DaoActionIdent, FnCallMetadata, ValidatorRef, ValueContainer},
};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
/// Wrapper around `TemplateActivity`.
pub enum Activity {
    Init,
    /// Contains only DaoActions.
    DaoActivity(TemplateActivity),
    /// Contains only FnCall actions.
    FnCallActivity(TemplateActivity),
}

impl Activity {
    pub fn is_dao_activity(&self) -> bool {
        match self {
            Self::DaoActivity(_) => true,
            _ => false,
        }
    }
}

impl Activity {
    pub fn into_activity(self) -> Option<TemplateActivity> {
        match self {
            Activity::Init => None,
            Activity::DaoActivity(a) => Some(a),
            Activity::FnCallActivity(a) => Some(a),
        }
    }

    pub fn activity_as_ref(&self) -> Option<&TemplateActivity> {
        match self {
            &Activity::Init => None,
            &Activity::DaoActivity(ref a) => Some(a),
            &Activity::FnCallActivity(ref a) => Some(a),
        }
    }

    pub fn activity_as_mut(&mut self) -> Option<&mut TemplateActivity> {
        match self {
            Activity::Init => None,
            Activity::DaoActivity(a) => Some(a),
            Activity::FnCallActivity(a) => Some(a),
        }
    }
}

/// Defines activity relation to workflow finish.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Clone, Copy)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum Terminality {
    /// Is not "end" activity.
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
                ActionData::Action(a) => Some(a.name.clone()),
                _ => None,
            },
            None => None,
        }
    }
}

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
    pub values: ObjectValues,
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
                FnCallIdType::Dynamic(_, m) => *l0.0 == *m,
                FnCallIdType::StandardStatic(s) => *l0 == *s,
                FnCallIdType::StandardDynamic(_, m) => *l0.0 == *m,
            },
            (Self::FnCall(_), ActionData::Action(_)) => false,
            _ => false,
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum Instruction {
    DeleteKey(String),
    DeleteKeyGlobal(String),
    /// User/Source provided value.
    StoreDynValue(String, ArgSrc),
    StoreValue(String, DataType),
    StoreValueGlobal(String, DataType),
    StoreFnCallResult(String, DataTypeDef),
    StoreFnCallResultGlobal(String, DataTypeDef),
    StoreWorkflow,
    /// TODO: Not implemented yet.
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
    /// Replaces all StoryDynValue variants with StoreValue variants.
    /// Returns `Err(())` in case users's input structure is not correct.
    pub fn bind_and_convert<T: AsRef<[DataType]>>(
        &mut self,
        value_source: &ValueContainer<T>,
        user_input: &Vec<Vec<DataType>>,
    ) -> Result<(), ()> {
        // TODO: Improve.
        for ins in self.instructions.iter_mut() {
            if let Instruction::StoreDynValue(string, arg_src) = ins {
                let value = match arg_src {
                    ArgSrc::User(id) => user_input
                        .get(0)
                        .ok_or(())?
                        .get(*id as usize)
                        .ok_or(())?
                        .clone(),
                    _ => get_value_from_source(arg_src, value_source).map_err(|_| ())?,
                };
                *ins = Instruction::StoreValue(string.clone(), value);
            }
        }

        Ok(())
    }

    // TODO: Finish all variants.
    pub fn execute(
        self,
        fn_result_val: Vec<u8>,
        storage: &mut Option<&mut StorageBucket>,
        global_storage: &mut StorageBucket,
        new_template: &mut Option<(
            Template,
            Vec<FnCallId>,
            Vec<Vec<FnCallMetadata>>,
            Vec<MethodName>,
            Vec<Vec<FnCallMetadata>>,
        )>,
    ) -> Result<(), ()> {
        let mut i = 0;
        while i < self.instructions.len() {
            match &self.instructions[i] {
                Instruction::DeleteKey(key) => {
                    storage.as_mut().unwrap().remove_data(&key);
                }
                Instruction::DeleteKeyGlobal(key) => {
                    global_storage.remove_data(&key);
                }
                Instruction::StoreValue(key, value) => {
                    storage.as_mut().unwrap().add_data(&key, &value)
                }
                Instruction::StoreValueGlobal(key, value) => global_storage.add_data(&key, &value),
                Instruction::StoreFnCallResult(key, type_def) => {
                    let result = self.deser_datatype_from_slice(&type_def, &fn_result_val)?;
                    storage.as_mut().unwrap().add_data(&key, &result);
                }
                Instruction::StoreFnCallResultGlobal(key, type_def) => {
                    let result = self.deser_datatype_from_slice(&type_def, &fn_result_val)?;
                    global_storage.add_data(&key, &result);
                }
                Instruction::StoreWorkflow => {
                    let (workflow, fncalls, fncall_metadata, std_fncalls, std_fncall_metadata): (
                        Template,
                        Vec<FnCallId>,
                        Vec<Vec<FnCallMetadata>>,
                        Vec<MethodName>,
                        Vec<Vec<FnCallMetadata>>,
                    ) = serde_json::from_slice(&fn_result_val).unwrap();

                    *new_template = Some((
                        workflow,
                        fncalls,
                        fncall_metadata,
                        std_fncalls,
                        std_fncall_metadata,
                    ))
                }
                Instruction::StoreDynValue(_, _) => Err(())?,
                Instruction::StoreExpression(_, _) => unimplemented!(),
                // TODO: finish.
                Instruction::Cond(cond) => {
                    i = cond.eval(&[]).map_err(|_| ())? as usize;
                    continue;
                }
            }

            i += 1;
        }

        Ok(())
    }

    fn deser_datatype_from_slice(
        &self,
        type_def: &DataTypeDef,
        promise_result_data: &Vec<u8>,
    ) -> Result<DataType, ()> {
        match type_def {
            DataTypeDef::String(_) => {
                let value = serde_json::from_slice::<String>(promise_result_data);
                Ok(DataType::String(value.map_err(|_| ())?))
            }
            DataTypeDef::Bool(_) => {
                let value = serde_json::from_slice::<bool>(promise_result_data);
                Ok(DataType::Bool(value.map_err(|_| ())?))
            }
            DataTypeDef::U64(_) => {
                let value = serde_json::from_slice::<u64>(promise_result_data);
                Ok(DataType::U64(value.map_err(|_| ())?))
            }
            DataTypeDef::U128(_) => {
                let value = serde_json::from_slice::<U128>(promise_result_data);
                Ok(DataType::U128(value.map_err(|_| ())?))
            }
            DataTypeDef::VecString => {
                let value = serde_json::from_slice::<Vec<String>>(promise_result_data);
                Ok(DataType::VecString(value.map_err(|_| ())?))
            }
            DataTypeDef::VecU64 => {
                let value = serde_json::from_slice::<Vec<u64>>(promise_result_data);
                Ok(DataType::VecU64(value.map_err(|_| ())?))
            }
            DataTypeDef::VecU128 => {
                let value = serde_json::from_slice::<Vec<U128>>(promise_result_data);
                Ok(DataType::VecU128(value.map_err(|_| ())?))
            }
            _ => Err(()),
        }
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

#[cfg(test)]
mod test {
    use crate::{
        storage::StorageBucket,
        types::DataType,
        workflow::types::{ArgSrc, ValueContainer},
    };

    use super::{Instruction, Postprocessing};

    // TODO: Extend.
    #[test]
    fn simple_postprocessing() {
        let user_input = vec![vec![DataType::String("value_1".into()), DataType::U64(420)]];
        let mut pp = Postprocessing {
            storage_key: "key".into(),
            instructions: vec![
                Instruction::StoreDynValue("skey_1".into(), ArgSrc::User(0)),
                Instruction::StoreDynValue("skey_2".into(), ArgSrc::User(1)),
            ],
        };

        let mut global_storage = StorageBucket::new(b"global".to_vec());

        let dao_consts = Box::new(|id: u8| match id {
            0 => Some(DataType::String("neardao.near".into())),
            _ => None,
        });

        let source = ValueContainer {
            dao_consts: &dao_consts,
            tpl_consts: &[],
            settings_consts: &[],
            activity_shared_consts: None,
            action_proposal_consts: None,
            storage: None,
            global_storage: &mut global_storage,
        };

        pp.bind_and_convert(&source, &user_input)
            .expect("PP - bind_and_convert failed.");

        let expected_pp_binded = Postprocessing {
            storage_key: "key".into(),
            instructions: vec![
                Instruction::StoreValue("skey_1".into(), DataType::String("value_1".into())),
                Instruction::StoreValue("skey_2".into(), DataType::U64(420)),
            ],
        };

        assert_eq!(pp, expected_pp_binded);
    }
}
