use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
};

use crate::types::DataType;

type ArgId = u8;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum ExprTerm {
    Value(DataType),
    Arg(ArgId),
    FnCall(FnName, (u8, u8)),
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum FnName {
    Concat,
    InString,
    InArray,
    ArrayAtIdx,
    ArrayRemove,
    ArrayPush,
    ArrayPop, // TODO remove?? when we have array_remove
    ArrayMerge,
    ArrayLen,
}

// Recursive structure does not work with deserializer
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct TExpr {
    pub operators: Vec<Op>,
    pub terms: Vec<ExprTerm>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Condition {
    pub expr: EExpr,
    pub true_path: u8,
    pub false_path: u8,
}

impl Condition {
    pub fn eval(&self, args: &[DataType]) -> u8 {
        if let DataType::Bool(v) = self.expr.eval(args) {
            match v {
                true => self.true_path,
                false => self.false_path,
            }
        } else {
            panic!("{}", "Cond expr must return bool");
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Op {
    pub operands_ids: [u8; 2],
    pub op_type: EOp,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum EExpr {
    Aritmetic(TExpr),
    Boolean(TExpr),
    String(TExpr),
    Fn(FnName),
    Value(DataType),
}

impl EExpr {
    pub fn eval(&self, args: &[DataType]) -> DataType {
        match self {
            EExpr::Aritmetic(e) | EExpr::Boolean(e) | EExpr::String(e) => {
                self.eval_expr(e, args).unwrap()
            }
            EExpr::Fn(fn_name) => self.eval_fn(fn_name, args),
            EExpr::Value(v) => v.clone(),
        }
    }

    pub fn eval_expr(&self, expr: &TExpr, args: &[DataType]) -> Result<DataType, String> {
        let mut results = Vec::with_capacity(expr.terms.len());
        for op in expr.operators.iter() {
            let temp_res = match &op.op_type {
                EOp::Log(_) => {
                    let (lhs, rhs) = (
                        results.get(op.operands_ids[0] as usize).unwrap(),
                        results.get(op.operands_ids[1] as usize).unwrap(),
                    );
                    op.op_type.operate(lhs, rhs)
                }
                _ => {
                    let (lhs, rhs) = (
                        expr.terms.get(op.operands_ids[0] as usize).unwrap(),
                        expr.terms.get(op.operands_ids[1] as usize).unwrap(),
                    );

                    let lhs = match lhs {
                        ExprTerm::Value(v) => v.clone(),
                        ExprTerm::Arg(id) => args[*id as usize].clone(),
                        ExprTerm::FnCall(fn_name, (li, ui)) => {
                            self.eval_fn(fn_name, &args[*li as usize..=*ui as usize])
                        }
                    };

                    let rhs = match rhs {
                        ExprTerm::Value(v) => v.clone(),
                        ExprTerm::Arg(id) => args[*id as usize].clone(),
                        ExprTerm::FnCall(fn_name, (li, ui)) => {
                            self.eval_fn(fn_name, &args[*li as usize..=*ui as usize])
                        }
                    };

                    op.op_type.operate(&lhs, &rhs)
                }
            };
            results.push(temp_res);
        }

        Ok(results.pop().unwrap())
    }

    fn eval_fn(&self, fn_name: &FnName, args: &[DataType]) -> DataType {
        match fn_name {
            FnName::Concat => {
                let mut result = String::with_capacity(64);

                for i in 0..args.len() {
                    // cannot be None coz we iterate by the array
                    match args.get(i).unwrap() {
                        DataType::String(ref v) => result.push_str(v),
                        _ => panic!("{}", "Expected DataType::VecString"),
                    };
                }
                DataType::String(result)
            }
            FnName::ArrayMerge => match &args[0] {
                DataType::String(s) => {
                    let mut result = Vec::with_capacity(args.len());
                    for arg in args.iter() {
                        result.push(
                            arg.clone()
                                .try_into_string()
                                .expect("Expected string datatype"),
                        );
                    }
                    DataType::VecString(result)
                }
                _ => panic!("Array merge is not yet supported for other types"),
            },
            /*
            FnName::InString => {
                let arg1 = parse_fn_arg(&args, 0, vmap)?;
                let arg2 = parse_fn_arg(&args, 1, vmap)?;

                match (arg1, arg2) {
                    (Some(Value::String(value)), Some(Value::String(haystack))) => {
                        Ok(Value::Boolean(haystack.contains(value)))
                    }
                    _ => Err(RuntimeErr::InvalidFunctionArgument),
                }
            }
            FnName::InArray => {
                let arg1 = parse_fn_arg(&args, 0, vmap)?;
                let arg2 = parse_fn_arg(&args, 1, vmap)?;
                match (arg1, arg2) {
                    (Some(Value::String(value)), Some(Value::ArrString(arr))) => {
                        Ok(Value::Boolean(arr.contains(value)))
                    }
                    (Some(Value::Integer(value)), Some(Value::ArrInteger(arr))) => {
                        Ok(Value::Boolean(arr.contains(value)))
                    }
                    _ => Err(RuntimeErr::InvalidFunctionArgument),
                }
            }
            FnName::ArrayAtIdx => {
                let arg1 = parse_fn_arg(&args, 0, vmap)?;
                let arg2 = parse_fn_arg(&args, 1, vmap)?;
                match (arg1, arg2) {
                    (Some(Value::Integer(value)), Some(Value::ArrString(arr))) => {
                        let result = if let Some(v) = arr.get(*value as usize) {
                            Value::String(v.into())
                        } else {
                            Value::Null
                        };
                        Ok(result)
                    }
                    (Some(Value::Integer(value)), Some(Value::ArrInteger(arr))) => {
                        let result = if let Some(v) = arr.get(*value as usize) {
                            Value::Integer(*v)
                        } else {
                            Value::Null
                        };
                        Ok(result)
                    }
                    _ => Err(RuntimeErr::InvalidFunctionArgument),
                }
            }
            FnName::ArrayPush => {
                //array insert
                let arg1 = parse_fn_arg(&args, 0, vmap)?;
                let arg2 = parse_fn_arg(&args, 1, vmap)?;
                match (arg1, arg2) {
                    (Some(Value::String(value)), Some(Value::ArrString(arr))) => {
                        //TODO: mutate or let it clone new vec?
                        let mut new_arr = arr.to_owned();
                        new_arr.push(value.into());
                        Ok(Value::ArrString(new_arr))
                    }
                    (Some(Value::Integer(value)), Some(Value::ArrInteger(arr))) => {
                        //TODO: mutate or let it clone new vec?
                        let mut new_arr = arr.to_owned();
                        new_arr.push(*value);
                        Ok(Value::ArrInteger(new_arr))
                    }
                    _ => Err(RuntimeErr::InvalidFunctionArgument),
                }
            }
            //Array remove instead ??
            FnName::ArrayPop => {
                match parse_fn_arg(&args, 0, vmap)? {
                    Some(Value::ArrString(arr)) => {
                        //TODO: mutate or let it clone new vec?
                        let mut new_arr = arr.to_owned();
                        new_arr.pop();
                        Ok(Value::ArrString(new_arr))
                    }
                    Some(Value::ArrInteger(arr)) => {
                        //TODO: mutate or let it clone new vec?
                        let mut new_arr = arr.to_owned();
                        new_arr.pop();
                        Ok(Value::ArrInteger(new_arr))
                    }
                    _ => Err(RuntimeErr::InvalidFunctionArgument),
                }
            }
              */
            // TODO array len?
            _ => panic!("{}", "Fn eval error"),
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum AriOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum RelOp {
    Eqs,
    NEqs,
    Gt,
    Lt,
    GtE,
    LtE,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum LogOp {
    And,
    Or,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum EOp {
    Ari(AriOp),
    Log(LogOp),
    Rel(RelOp),
}

impl EOp {
    pub fn operate(&self, arg1: &DataType, arg2: &DataType) -> DataType {
        match self {
            EOp::Ari(o) => {
                let (lhs, rhs) = match (arg1, arg2) {
                    (DataType::U8(lhs), DataType::U8(rhs)) => (*lhs, *rhs),
                    _ => panic!("Invalid operands for aritmetic operation"),
                };

                let result = match o {
                    AriOp::Add => lhs + rhs,
                    AriOp::Subtract => lhs - rhs,
                    AriOp::Multiply => lhs * rhs,
                    AriOp::Divide => lhs / rhs,
                    AriOp::Modulo => lhs % rhs,
                };

                DataType::U8(result)
            }
            EOp::Ari(o) => {
                let (lhs, rhs) = match (arg1, arg2) {
                    (DataType::U128(lhs), DataType::U128(rhs)) => (*lhs, *rhs),
                    _ => panic!("Invalid operands for aritmetic operation"),
                };

                let result = match o {
                    AriOp::Add => lhs.0 + rhs.0,
                    AriOp::Subtract => lhs.0 - rhs.0,
                    AriOp::Multiply => lhs.0 * rhs.0,
                    AriOp::Divide => lhs.0 / rhs.0,
                    AriOp::Modulo => lhs.0 % rhs.0,
                };

                DataType::U128(result.into())
            }
            EOp::Rel(o) => match (arg1, arg2) {
                (DataType::Bool(lhs), DataType::Bool(rhs)) => match o {
                    RelOp::Eqs => DataType::Bool(*lhs == *rhs),
                    RelOp::NEqs => DataType::Bool(*lhs != *rhs),
                    _ => panic!("Invalid operation"),
                },
                (DataType::U8(lhs), DataType::U8(rhs)) => match o {
                    RelOp::Eqs => DataType::Bool(lhs == rhs),
                    RelOp::NEqs => DataType::Bool(lhs != rhs),
                    RelOp::Gt => DataType::Bool(lhs > rhs),
                    RelOp::Lt => DataType::Bool(lhs < rhs),
                    RelOp::GtE => DataType::Bool(lhs >= rhs),
                    RelOp::LtE => DataType::Bool(lhs <= rhs),
                    _ => panic!("Invalid operation"),
                },
                (DataType::U16(lhs), DataType::U16(rhs)) => match o {
                    RelOp::Eqs => DataType::Bool(lhs == rhs),
                    RelOp::NEqs => DataType::Bool(lhs != rhs),
                    RelOp::Gt => DataType::Bool(lhs > rhs),
                    RelOp::Lt => DataType::Bool(lhs < rhs),
                    RelOp::GtE => DataType::Bool(lhs >= rhs),
                    RelOp::LtE => DataType::Bool(lhs <= rhs),
                    _ => panic!("Invalid operation"),
                },
                (DataType::U128(lhs), DataType::U128(rhs)) => match o {
                    RelOp::Eqs => DataType::Bool(lhs == rhs),
                    RelOp::NEqs => DataType::Bool(lhs != rhs),
                    RelOp::Gt => DataType::Bool(lhs.0 > rhs.0),
                    RelOp::Lt => DataType::Bool(lhs.0 < rhs.0),
                    RelOp::GtE => DataType::Bool(lhs.0 >= rhs.0),
                    RelOp::LtE => DataType::Bool(lhs.0 <= rhs.0),
                    _ => panic!("Invalid operation"),
                },
                (DataType::String(lhs), DataType::String(rhs)) => match o {
                    RelOp::Eqs => DataType::Bool(*lhs == *rhs),
                    RelOp::NEqs => DataType::Bool(*lhs != *rhs),
                    RelOp::Gt => DataType::Bool(*lhs > *rhs),
                    RelOp::Lt => DataType::Bool(*lhs < *rhs),
                    RelOp::GtE => DataType::Bool(*lhs >= *rhs),
                    RelOp::LtE => DataType::Bool(*lhs <= *rhs),
                    _ => panic!("Invalid operation"),
                },
                // TODO: which operations
                //(DataType::VecString(lhs), DataType::VecString(rhs)) => match o {
                //    _ => panic!("Invalid operation"),
                //},
                //(DataType::ArrInteger(lhs), DataType::ArrInteger(rhs)) => match o {
                //    _ => panic!("Invalid operation"),
                //},
                _ => panic!("Invalid operand types for this operation"),
            },
            EOp::Log(o) => match (arg1, arg2) {
                (DataType::Bool(lhs), DataType::Bool(rhs)) => match o {
                    LogOp::And => DataType::Bool(*lhs && *rhs),
                    LogOp::Or => DataType::Bool(*lhs || *rhs),
                    _ => panic!("Invalid operation"),
                },
                _ => panic!("Invalid operand tyes for this operation"),
            },
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        expression::{LogOp, Op},
        types::DataType,
    };

    use super::{EExpr, EOp, ExprTerm, FnName, RelOp, TExpr};

    #[test]
    pub fn expr_simple_1() {
        //TEST CASE
        //"1 > 2"

        let mut args = vec![DataType::U8(1), DataType::U8(2)];

        let expr = EExpr::Boolean(TExpr {
            operators: vec![Op {
                operands_ids: [0, 1],
                op_type: EOp::Rel(RelOp::Gt),
            }],
            terms: vec![ExprTerm::Arg(0), ExprTerm::Arg(1)],
        });

        let result = expr.eval(&mut args);
        let expected_result = DataType::Bool(false);

        assert_eq!(result, expected_result);
    }

    #[test]
    pub fn expr_fn_concat() {
        //TEST CASE
        //string = concat(["a", "b", "c"]) + "_group" //last one is binded

        let mut args = vec![
            DataType::String("a".into()),
            DataType::String("b".into()),
            DataType::String("c".into()),
            DataType::String("_group".into()),
        ];

        let expr = EExpr::Fn(FnName::Concat);

        let result = expr.eval(&mut args);
        let expected_result = DataType::String("abc_group".into());

        assert_eq!(result, expected_result);
    }
    #[test]
    pub fn expr_fn_concat_in_cond() {
        //TEST CASE
        //"abc_group" == concat(["a", "b", "c"]) + "_group" //last one is binded

        let mut args = vec![
            DataType::String("a".into()),
            DataType::String("b".into()),
            DataType::String("c".into()),
            DataType::String("_group".into()),
            DataType::String("abc_group".into()),
        ];

        let expr = EExpr::Boolean(TExpr {
            operators: vec![Op {
                operands_ids: [0, 1],
                op_type: EOp::Rel(RelOp::Eqs),
            }],
            terms: vec![ExprTerm::Arg(4), ExprTerm::FnCall(FnName::Concat, (0, 3))],
        });

        let result = expr.eval(&mut args);
        let expected_result = DataType::Bool(true);

        assert_eq!(result, expected_result);
    }

    #[test]
    pub fn expr_fn_concat_in_cond_or() {
        //TEST CASE
        //"abc_group" == concat(["a", "b", "c"]) + "_group" || 1 > 2  //last one is binded

        let mut args = vec![
            DataType::String("a".into()),
            DataType::String("b".into()),
            DataType::String("c".into()),
            DataType::String("_group".into()),
            DataType::String("abc_group".into()),
            DataType::U8(1),
            DataType::U8(2),
        ];

        let expr = EExpr::Boolean(TExpr {
            operators: vec![
                Op {
                    operands_ids: [0, 1],
                    op_type: EOp::Rel(RelOp::Eqs),
                },
                Op {
                    operands_ids: [2, 3],
                    op_type: EOp::Rel(RelOp::Gt),
                },
                Op {
                    operands_ids: [0, 1],
                    op_type: EOp::Log(LogOp::Or),
                },
            ],
            terms: vec![
                ExprTerm::Arg(4),
                ExprTerm::FnCall(FnName::Concat, (0, 3)),
                ExprTerm::Arg(5),
                ExprTerm::Arg(6),
            ],
        });

        let result = expr.eval(&mut args);
        let expected_result = DataType::Bool(true);

        assert_eq!(result, expected_result);
    }
}
