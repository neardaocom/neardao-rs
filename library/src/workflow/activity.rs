use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    json_types::U128,
    serde::{Deserialize, Serialize},
    serde_json,
};

use crate::{
    functions::binding::get_value_from_source,
    interpreter::{condition::Condition, expression::EExpr},
    storage::StorageBucket,
    types::datatype::{Datatype, Value},
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
        matches!(self, Self::DaoActivity(_))
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
            Activity::Init => None,
            Activity::DaoActivity(ref a) => Some(a),
            Activity::FnCallActivity(ref a) => Some(a),
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
                ActionData::Action(a) => Some(a.name),
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
                FnCallIdType::Dynamic(_, m) => *l0.0.as_str() == *m,
                FnCallIdType::StandardStatic(s) => *l0 == *s,
                FnCallIdType::StandardDynamic(_, m) => *l0.0.as_str() == *m,
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
    StoreValue(String, Value),
    StoreValueGlobal(String, Value),
    StoreFnCallResult(String, Datatype),
    StoreFnCallResultGlobal(String, Datatype),
    StoreWorkflow,
    /// Stores expression
    /// 2th param defines if FnCallResult is required and what to deserialize it to.
    /// FnCall result will always be as last arg in values.
    StoreExpression(String, Vec<ArgSrc>, EExpr, Option<Datatype>),
    StoreExpressionGlobal(String, Vec<ArgSrc>, EExpr, Option<Datatype>),
    StoreExpressionBinded(String, Vec<Value>, EExpr, Option<Datatype>),
    StoreExpressionGlobalBinded(String, Vec<Value>, EExpr, Option<Datatype>),
    /// Conditional Jump.
    /// 2th param defines if FnCallResult is required and what to deserialize it to.
    /// FnCall result will always be as last arg in values.
    Cond(Vec<ArgSrc>, Condition, Option<Datatype>),
    CondBinded(Vec<Value>, Condition, Option<Datatype>),
    Jump(u8),
    None,
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
    /// Same is valid for variants with opposite *Binded
    /// Supposed to be called before dispatching FnCall action.
    /// Returns `Err(())` in case users's input structure is not correct.
    #[allow(clippy::result_unit_err)]
    pub fn bind_instructions<T: AsRef<[Value]>>(
        &mut self,
        value_source: &ValueContainer<T>,
        user_input: &[Vec<Value>],
    ) -> Result<(), ()> {
        // TODO: Improve.
        for ins in self.instructions.iter_mut() {
            match ins {
                Instruction::StoreDynValue(string, arg_src) => {
                    let value = bind_value(arg_src, user_input, value_source)?;
                    *ins = Instruction::StoreValue(string.clone(), value);
                }
                Instruction::Cond(arg_src, cond, required_fncall_result) => {
                    let mut values = Vec::with_capacity(arg_src.len());
                    for src in arg_src.iter() {
                        let value = bind_value(src, user_input, value_source)?;
                        values.push(value);
                    }
                    *ins = Instruction::CondBinded(
                        values,
                        cond.clone(),
                        required_fncall_result.take(),
                    );
                }
                Instruction::StoreExpression(key, arg_src, expr, required_fncall_result) => {
                    let mut values = Vec::with_capacity(arg_src.len());
                    for src in arg_src.iter() {
                        let value = bind_value(src, user_input, value_source)?;
                        values.push(value);
                    }

                    *ins = Instruction::StoreExpressionBinded(
                        key.clone(),
                        values,
                        expr.clone(),
                        required_fncall_result.take(),
                    );
                }
                Instruction::StoreExpressionGlobal(key, arg_src, expr, required_fncall_result) => {
                    let mut values = Vec::with_capacity(arg_src.len());
                    for src in arg_src.iter() {
                        let value = bind_value(src, user_input, value_source)?;
                        values.push(value);
                    }

                    *ins = Instruction::StoreExpressionGlobalBinded(
                        key.clone(),
                        values,
                        expr.clone(),
                        required_fncall_result.take(),
                    );
                }
                _ => continue,
            }
        }

        Ok(())
    }
    /// Executes postprocessing script.
    #[allow(clippy::type_complexity)]
    #[allow(clippy::result_unit_err)]
    pub fn execute(
        mut self,
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
            // Replace/Swap to avoid cloning.
            let mut ins = std::mem::replace(&mut self.instructions[i], Instruction::None);
            match &mut ins {
                Instruction::DeleteKey(key) => {
                    storage.as_mut().unwrap().remove_data(key);
                }
                Instruction::DeleteKeyGlobal(key) => {
                    global_storage.remove_data(key);
                }
                Instruction::StoreValue(key, value) => {
                    storage.as_mut().unwrap().add_data(key, value)
                }
                Instruction::StoreValueGlobal(key, value) => global_storage.add_data(key, value),
                Instruction::StoreFnCallResult(key, type_def) => {
                    let result = self.deser_datatype_from_slice(type_def, &fn_result_val)?;
                    storage.as_mut().unwrap().add_data(key, &result);
                }
                Instruction::StoreFnCallResultGlobal(key, type_def) => {
                    let result = self.deser_datatype_from_slice(type_def, &fn_result_val)?;
                    global_storage.add_data(key, &result);
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
                Instruction::StoreExpression(_, _, _, _) => return Err(()),
                Instruction::StoreExpressionGlobal(_, _, _, _) => return Err(()),
                Instruction::CondBinded(values, cond, required_fncall_result) => {
                    // Bind FnCall result to values in condition.
                    if let Some(type_def) = required_fncall_result {
                        let result = self.deser_datatype_from_slice(type_def, &fn_result_val)?;
                        values.push(result);
                    }

                    let next_ins = cond.eval(values.as_slice()).map_err(|_| ())? as usize;
                    // In case this condition is evaluated again we want to restore it back.
                    values.pop();
                    std::mem::swap(&mut self.instructions[i], &mut ins);
                    i = next_ins;
                    continue;
                }
                Instruction::Jump(idx) => {
                    let next_ins = *idx as usize;
                    std::mem::swap(&mut self.instructions[i], &mut ins);
                    i = next_ins;
                    continue;
                }
                Instruction::StoreDynValue(_, _) => return Err(()),
                Instruction::Cond(_, _, _) => return Err(()),
                Instruction::None => continue,
                Instruction::StoreExpressionBinded(key, values, expr, required_fncall_result) => {
                    // Bind FnCall result to values in condition.
                    if let Some(type_def) = required_fncall_result {
                        let result = self.deser_datatype_from_slice(type_def, &fn_result_val)?;
                        values.push(result);
                    }

                    let result = expr.eval(values.as_slice()).map_err(|_| ())?;
                    storage.as_mut().unwrap().add_data(key, &result);

                    values.pop();
                }
                Instruction::StoreExpressionGlobalBinded(
                    key,
                    values,
                    expr,
                    required_fncall_result,
                ) => {
                    // Bind FnCall result to values in condition.
                    if let Some(type_def) = required_fncall_result {
                        let result = self.deser_datatype_from_slice(type_def, &fn_result_val)?;
                        values.push(result);
                    }

                    let result = expr.eval(values.as_slice()).map_err(|_| ())?;
                    global_storage.add_data(key, &result);

                    values.pop();
                }
            }
            // Swap instruction back.
            std::mem::swap(&mut self.instructions[i], &mut ins);

            i += 1;
        }

        Ok(())
    }

    #[allow(clippy::result_unit_err)]
    fn deser_datatype_from_slice(
        &self,
        type_def: &Datatype,
        promise_result_data: &[u8],
    ) -> Result<Value, ()> {
        match type_def {
            Datatype::String(_) => {
                let value = serde_json::from_slice::<String>(promise_result_data);
                Ok(Value::String(value.map_err(|_| ())?))
            }
            Datatype::Bool(_) => {
                let value = serde_json::from_slice::<bool>(promise_result_data);
                Ok(Value::Bool(value.map_err(|_| ())?))
            }
            Datatype::U64(_) => {
                let value = serde_json::from_slice::<u64>(promise_result_data);
                Ok(Value::U64(value.map_err(|_| ())?))
            }
            Datatype::U128(_) => {
                let value = serde_json::from_slice::<U128>(promise_result_data);
                Ok(Value::U128(value.map_err(|_| ())?))
            }
            Datatype::VecString => {
                let value = serde_json::from_slice::<Vec<String>>(promise_result_data);
                Ok(Value::VecString(value.map_err(|_| ())?))
            }
            Datatype::VecU64 => {
                let value = serde_json::from_slice::<Vec<u64>>(promise_result_data);
                Ok(Value::VecU64(value.map_err(|_| ())?))
            }
            Datatype::VecU128 => {
                let value = serde_json::from_slice::<Vec<U128>>(promise_result_data);
                Ok(Value::VecU128(value.map_err(|_| ())?))
            }
            _ => Err(()),
        }
    }
}

#[allow(clippy::result_unit_err)]
fn bind_value<T: AsRef<[Value]>>(
    arg_src: &ArgSrc,
    user_input: &[Vec<Value>],
    value_source: &ValueContainer<T>,
) -> Result<Value, ()> {
    let value = match arg_src {
        ArgSrc::UserObj(obj_id, arg_id) => user_input
            .get(*obj_id as usize)
            .ok_or(())?
            .get(*arg_id as usize)
            .ok_or(())?
            .clone(),
        _ => get_value_from_source(arg_src, value_source).map_err(|_| ())?,
    };
    Ok(value)
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
    use near_sdk::{serde_json, test_utils::VMContextBuilder, testing_env};

    use crate::{
        interpreter::{
            condition::Condition,
            expression::{AriOp, EExpr, EOp, ExprTerm, Op, RelOp, TExpr},
        },
        storage::StorageBucket,
        types::datatype::{Datatype, Value},
        workflow::types::{ArgSrc, ValueContainer},
    };

    use super::{Instruction, Postprocessing};

    // TODO: Come up with better test case than this one.
    /// Test case 1:
    /// User input values: "value_1", 420
    /// Assume FnCall result => string: registered/unregistered
    /// If registered, then global storage save 2 * 420,
    /// Else wf storage save "requires registration",
    #[test]
    fn postprocessing_simple_cond_1() {
        testing_env!(VMContextBuilder::new().build());
        let user_input = vec![vec![Value::String("value_1".into()), Value::U64(420)]];

        let mut pp = Postprocessing {
            storage_key: "key".into(),
            instructions: vec![
                Instruction::Cond(
                    vec![],
                    Condition {
                        expr: EExpr::String(TExpr {
                            operators: vec![Op {
                                operands_ids: [0, 1],
                                op_type: EOp::Rel(RelOp::Eqs),
                            }],
                            terms: vec![
                                ExprTerm::Arg(0),
                                ExprTerm::Value(Value::String("registered".into())),
                            ],
                        }),
                        true_path: 1,
                        false_path: 3,
                    },
                    Some(Datatype::String(false)),
                ),
                Instruction::StoreExpressionGlobal(
                    "skey_1".into(),
                    vec![ArgSrc::UserObj(0, 1)],
                    EExpr::Aritmetic(TExpr {
                        operators: vec![Op {
                            operands_ids: [0, 1],
                            op_type: EOp::Ari(AriOp::Multiply),
                        }],
                        terms: vec![ExprTerm::Arg(0), ExprTerm::Value(Value::U64(2))],
                    }),
                    None,
                ),
                Instruction::Jump(u8::MAX),
                Instruction::StoreValue(
                    "skey_2".into(),
                    Value::String("requires_registration".into()),
                ),
            ],
        };

        let mut global_storage = StorageBucket::new(b"global".to_vec());
        let mut storage = StorageBucket::new(b"key".to_vec());

        let dao_consts = Box::new(|id: u8| match id {
            0 => Some(Value::String("neardao.near".into())),
            _ => None,
        });

        let source = ValueContainer {
            dao_consts: &dao_consts,
            tpl_consts: &[],
            settings_consts: &[],
            activity_shared_consts: None,
            action_proposal_consts: None,
            storage: Some(&mut storage),
            global_storage: &mut global_storage,
        };

        pp.bind_instructions(&source, &user_input)
            .expect("PP - bind_and_convert failed.");

        let expected_pp_binded = Postprocessing {
            storage_key: "key".into(),
            instructions: vec![
                Instruction::CondBinded(
                    vec![],
                    Condition {
                        expr: EExpr::String(TExpr {
                            operators: vec![Op {
                                operands_ids: [0, 1],
                                op_type: EOp::Rel(RelOp::Eqs),
                            }],
                            terms: vec![
                                ExprTerm::Arg(0),
                                ExprTerm::Value(Value::String("registered".into())),
                            ],
                        }),
                        true_path: 1,
                        false_path: 3,
                    },
                    Some(Datatype::String(false)),
                ),
                Instruction::StoreExpressionGlobalBinded(
                    "skey_1".into(),
                    vec![Value::U64(420)],
                    EExpr::Aritmetic(TExpr {
                        operators: vec![Op {
                            operands_ids: [0, 1],
                            op_type: EOp::Ari(AriOp::Multiply),
                        }],
                        terms: vec![ExprTerm::Arg(0), ExprTerm::Value(Value::U64(2))],
                    }),
                    None,
                ),
                Instruction::Jump(u8::MAX),
                Instruction::StoreValue(
                    "skey_2".into(),
                    Value::String("requires_registration".into()),
                ),
            ],
        };

        assert_eq!(pp, expected_pp_binded);

        assert_eq!(global_storage.get_all_data().len(), 0);
        assert_eq!(storage.get_all_data().len(), 0);

        // Registered case

        let result = "registered".to_string();
        let result_raw = serde_json::to_string(&result).unwrap().into_bytes();

        assert!(pp
            .clone()
            .execute(
                result_raw,
                &mut Some(&mut storage),
                &mut global_storage,
                &mut None,
            )
            .is_ok());

        assert_eq!(
            global_storage.get_all_data(),
            vec![("skey_1".into(), Value::U64(2 * 420))]
        );

        assert_eq!(storage.get_all_data(), vec![]);

        // Unregistered case

        let result = "unregistered".to_string();
        let result_raw = serde_json::to_string(&result).unwrap().into_bytes();

        global_storage.remove_data(&"skey_1".into());

        assert_eq!(global_storage.get_all_data().len(), 0);
        assert_eq!(storage.get_all_data().len(), 0);

        assert!(pp
            .execute(
                result_raw,
                &mut Some(&mut storage),
                &mut global_storage,
                &mut None,
            )
            .is_ok());

        assert_eq!(global_storage.get_all_data(), vec![]);
        assert_eq!(
            storage.get_all_data(),
            vec![(
                "skey_2".into(),
                Value::String("requires_registration".into())
            )]
        );
    }
}
