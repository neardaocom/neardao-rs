use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    json_types::U128,
    serde::{Deserialize, Serialize},
    serde_json,
};

use crate::{
    functions::evaluation::eval,
    interpreter::expression::EExpr,
    storage::StorageBucket,
    types::{
        activity_input::ActivityInput,
        datatype::{Datatype, Value},
        error::SourceError,
        source::Source,
    },
    ProviderTemplateData,
};

use super::types::Instruction;

// TODO: Remove Debug in production.
/// Set of instructions executed after action.
/// It is usually used to save function call result data or save some data to the storage.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Postprocessing {
    pub instructions: Vec<Instruction>,
}

impl Postprocessing {
    // TODO: replace with PostprocessingError
    /// Replaces all StoryDynValue variants with StoreValue variants.
    /// Same is valid for variants with opposite *Binded
    /// Supposed to be called before dispatching FnCall action.
    /// Returns `Err(())` in case users's input structure is not correct.
    pub fn bind_instructions(
        &mut self,
        sources: &dyn Source,
        expressions: &[EExpr],
        user_input: &dyn ActivityInput,
    ) -> Result<(), SourceError> {
        // TODO: Improve.
        for ins in self.instructions.iter_mut() {
            match ins {
                Instruction::StoreDynValue(string, arg_src) => {
                    let value = eval(arg_src, sources, expressions, Some(user_input))
                        .expect("postprocessing eval value missing");
                    *ins = Instruction::StoreValue(string.clone(), value);
                }
                Instruction::Cond(arg_src, cond, required_fncall_result) => {
                    let mut values = Vec::with_capacity(arg_src.len());
                    for src in arg_src.iter() {
                        let value = eval(src, sources, expressions, Some(user_input))
                            .expect("postprocessing eval value missing");
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
                        let value = eval(src, sources, expressions, Some(user_input))
                            .expect("postprocessing eval value missing");
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
                        let value = eval(src, sources, expressions, Some(user_input))
                            .expect("postprocessing eval value missing");
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
    // TODO: Error handling
    /// Executes postprocessing script.
    pub fn execute(
        mut self,
        fn_result_val: Vec<u8>,
        mut storage: Option<&mut StorageBucket>,
        global_storage: &mut StorageBucket,
        new_template: &mut Option<ProviderTemplateData>,
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
                    let result = self.deser_datatype_from_slice(
                        type_def
                            .into_datatype_ref()
                            .expect("custom types are not suported yet"),
                        &fn_result_val,
                    )?;
                    storage.as_mut().unwrap().add_data(key, &result);
                }
                Instruction::StoreFnCallResultGlobal(key, type_def) => {
                    let result = self.deser_datatype_from_slice(
                        type_def
                            .into_datatype_ref()
                            .expect("custom types are not suported yet"),
                        &fn_result_val,
                    )?;
                    global_storage.add_data(key, &result);
                }
                Instruction::StoreWorkflow => {
                    let (workflow, fncalls, fncall_metadata): ProviderTemplateData =
                        serde_json::from_slice(&fn_result_val).unwrap();

                    *new_template = Some((workflow, fncalls, fncall_metadata))
                }
                Instruction::StoreExpression(_, _, _, _) => return Err(()),
                Instruction::StoreExpressionGlobal(_, _, _, _) => return Err(()),
                Instruction::CondBinded(values, cond, required_fncall_result) => {
                    // Bind FnCall result to values in condition.
                    if let Some(type_def) = required_fncall_result {
                        let result = self.deser_datatype_from_slice(
                            type_def
                                .into_datatype_ref()
                                .expect("custom types are not suported yet"),
                            &fn_result_val,
                        )?;
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
                        let result = self.deser_datatype_from_slice(
                            type_def
                                .into_datatype_ref()
                                .expect("custom types are not suported yet"),
                            &fn_result_val,
                        )?;
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
                        let result = self.deser_datatype_from_slice(
                            type_def
                                .into_datatype_ref()
                                .expect("custom types are not suported yet"),
                            &fn_result_val,
                        )?;
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

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use near_sdk::{serde_json, test_utils::VMContextBuilder, testing_env};

    use crate::{
        interpreter::{
            condition::Condition,
            expression::{AriOp, EExpr, EOp, ExprTerm, Op, RelOp, TExpr},
        },
        storage::StorageBucket,
        types::{
            activity_input::ActivityInput,
            datatype::{Datatype, Value},
            source::SourceMock,
        },
        workflow::{
            postprocessing::Postprocessing,
            types::{FnCallResultType, Instruction, Src, ValueSrc},
        },
    };

    // TODO: Come up with better test case than this one.
    /// Test case 1:
    /// User input values: "value_1", 420
    /// Assume FnCall result => string: registered/unregistered
    /// If registered, then global storage save 2 * 420.
    /// Else wf storage save "requires registration".
    #[test]
    fn postprocessing_simple_cond_1() {
        testing_env!(VMContextBuilder::new().build());

        let mut hm = HashMap::new();
        hm.set("key_1", Value::String("value_1".into()));
        hm.set("key_2", Value::U64(420));

        let user_input: Box<dyn ActivityInput> = Box::new(hm);

        let mut pp = Postprocessing {
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
                    Some(FnCallResultType::Datatype(Datatype::String(false))),
                ),
                Instruction::StoreExpressionGlobal(
                    "skey_1".into(),
                    vec![ValueSrc::Src(Src::User("key_2".into()))],
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

        let _dao_consts = Box::new(|id: u8| match id {
            0 => Some(Value::String("neardao.near".into())),
            _ => None,
        });

        let source = SourceMock { tpls: vec![] };

        pp.bind_instructions(&source, &[], user_input.as_ref())
            .expect("PP - bind_and_convert failed.");

        let expected_pp_binded = Postprocessing {
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
                    Some(FnCallResultType::Datatype(Datatype::String(false))),
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
                Some(&mut storage),
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
                Some(&mut storage),
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
