use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
};
// TODO: Remove all Debug in production!

use crate::types::datatype::Value;

use super::{error::EvalError, ArgId};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq))]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum ExprTerm {
    /// Specific Value.
    Value(Value),
    /// Argument id with Value.
    Arg(ArgId),
    /// Function with defined interval (inclusive) for arguments.
    Fn(FnName, (ArgId, ArgId)),
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq))]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum FnName {
    Concat,
    InString,
    InArray,
    ArrayAtIdx,
    ArrayRemove,
    ArrayPush,
    ArrayPop,
    ArrayLen,
    ArrayMerge,
    ValueExists,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct TExpr {
    pub operators: Vec<Op>,
    pub terms: Vec<ExprTerm>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Op {
    pub operands_ids: [u8; 2],
    pub op_type: EOp,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq))]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum EExpr {
    Aritmetic(TExpr),
    Boolean(TExpr),
    String(TExpr),
    Fn(FnName),
    Value(Value),
}

impl EExpr {
    pub fn eval(&self, args: &[Value]) -> Result<Value, EvalError> {
        match self {
            EExpr::Aritmetic(e) | EExpr::Boolean(e) | EExpr::String(e) => self.eval_expr(e, args),
            EExpr::Fn(fn_name) => self.eval_fn(fn_name, args),
            EExpr::Value(v) => Ok(v.clone()),
        }
    }

    pub fn eval_expr(&self, expr: &TExpr, args: &[Value]) -> Result<Value, EvalError> {
        let mut results = Vec::with_capacity(expr.terms.len());
        for op in expr.operators.iter() {
            let temp_res = match &op.op_type {
                EOp::Log(_) => {
                    let (lhs, rhs) = (
                        results.get(op.operands_ids[0] as usize).unwrap(),
                        results.get(op.operands_ids[1] as usize).unwrap(),
                    );
                    op.op_type.eval(lhs, rhs)?
                }
                _ => {
                    let (lhs, rhs) = (
                        expr.terms.get(op.operands_ids[0] as usize).unwrap(),
                        expr.terms.get(op.operands_ids[1] as usize).unwrap(),
                    );

                    let lhs = match lhs {
                        ExprTerm::Value(v) => v.clone(),
                        ExprTerm::Arg(id) => args[*id as usize].clone(),
                        ExprTerm::Fn(fn_name, (li, ui)) => {
                            self.eval_fn(fn_name, &args[*li as usize..=*ui as usize])?
                        }
                    };

                    let rhs = match rhs {
                        ExprTerm::Value(v) => v.clone(),
                        ExprTerm::Arg(id) => args[*id as usize].clone(),
                        ExprTerm::Fn(fn_name, (li, ui)) => {
                            self.eval_fn(fn_name, &args[*li as usize..=*ui as usize])?
                        }
                    };

                    op.op_type.eval(&lhs, &rhs)?
                }
            };
            results.push(temp_res);
        }

        Ok(results.pop().unwrap())
    }

    fn eval_fn(&self, fn_name: &FnName, args: &[Value]) -> Result<Value, EvalError> {
        match fn_name {
            FnName::Concat => {
                let mut result = String::with_capacity(64);

                for val in args.iter() {
                    // cannot be None coz we iterate by the array
                    match val {
                        Value::String(ref v) => result.push_str(v),
                        _ => return Err(EvalError::InvalidType),
                    };
                }
                Ok(Value::String(result))
            }
            FnName::ArrayMerge => match &args[0] {
                Value::String(_) => {
                    let mut result = Vec::with_capacity(args.len());
                    for arg in args.iter() {
                        result.push(
                            arg.clone()
                                .try_into_string()
                                .map_err(|_| EvalError::InvalidType)?,
                        );
                    }
                    Ok(Value::VecString(result))
                }
                _ => Err(EvalError::Unimplemented),
            },
            FnName::ValueExists => match &args.get(0) {
                Some(_) => Ok(Value::Bool(true)),
                None => Ok(Value::Bool(false)),
            },
            _ => Err(EvalError::Unimplemented),
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq))]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum AriOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq))]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum RelOp {
    Eqs,
    NEqs,
    Gt,
    Lt,
    GtE,
    LtE,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq))]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum LogOp {
    And,
    Or,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq))]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum EOp {
    Ari(AriOp),
    Log(LogOp),
    Rel(RelOp),
}

impl EOp {
    pub fn eval(&self, arg1: &Value, arg2: &Value) -> Result<Value, EvalError> {
        match self {
            EOp::Ari(o) => match (arg1, arg2) {
                (Value::U64(lhs), Value::U64(rhs)) => {
                    let result = match o {
                        AriOp::Add => lhs + rhs,
                        AriOp::Subtract => lhs - rhs,
                        AriOp::Multiply => lhs * rhs,
                        AriOp::Divide => lhs / rhs,
                        AriOp::Modulo => lhs % rhs,
                    };

                    Ok(Value::U64(result))
                }
                (Value::U128(lhs), Value::U128(rhs)) => {
                    let result = match o {
                        AriOp::Add => lhs.0 + rhs.0,
                        AriOp::Subtract => lhs.0 - rhs.0,
                        AriOp::Multiply => lhs.0 * rhs.0,
                        AriOp::Divide => lhs.0 / rhs.0,
                        AriOp::Modulo => lhs.0 % rhs.0,
                    };

                    Ok(Value::U128(result.into()))
                }
                _ => panic!("Invalid operands for aritmetic operation"),
            },
            EOp::Rel(o) => match (arg1, arg2) {
                (Value::Bool(lhs), Value::Bool(rhs)) => match o {
                    RelOp::Eqs => Ok(Value::Bool(*lhs == *rhs)),
                    RelOp::NEqs => Ok(Value::Bool(*lhs != *rhs)),
                    _ => Err(EvalError::Unimplemented),
                },
                (Value::U64(lhs), Value::U64(rhs)) => match o {
                    RelOp::Eqs => Ok(Value::Bool(lhs == rhs)),
                    RelOp::NEqs => Ok(Value::Bool(lhs != rhs)),
                    RelOp::Gt => Ok(Value::Bool(lhs > rhs)),
                    RelOp::Lt => Ok(Value::Bool(lhs < rhs)),
                    RelOp::GtE => Ok(Value::Bool(lhs >= rhs)),
                    RelOp::LtE => Ok(Value::Bool(lhs <= rhs)),
                },
                (Value::U128(lhs), Value::U128(rhs)) => match o {
                    RelOp::Eqs => Ok(Value::Bool(lhs == rhs)),
                    RelOp::NEqs => Ok(Value::Bool(lhs != rhs)),
                    RelOp::Gt => Ok(Value::Bool(lhs.0 > rhs.0)),
                    RelOp::Lt => Ok(Value::Bool(lhs.0 < rhs.0)),
                    RelOp::GtE => Ok(Value::Bool(lhs.0 >= rhs.0)),
                    RelOp::LtE => Ok(Value::Bool(lhs.0 <= rhs.0)),
                },
                (Value::String(lhs), Value::String(rhs)) => match o {
                    RelOp::Eqs => Ok(Value::Bool(*lhs == *rhs)),
                    RelOp::NEqs => Ok(Value::Bool(*lhs != *rhs)),
                    RelOp::Gt => Ok(Value::Bool(*lhs > *rhs)),
                    RelOp::Lt => Ok(Value::Bool(*lhs < *rhs)),
                    RelOp::GtE => Ok(Value::Bool(*lhs >= *rhs)),
                    RelOp::LtE => Ok(Value::Bool(*lhs <= *rhs)),
                },
                _ => Err(EvalError::InvalidType),
            },
            EOp::Log(o) => match (arg1, arg2) {
                (Value::Bool(lhs), Value::Bool(rhs)) => match o {
                    LogOp::And => Ok(Value::Bool(*lhs && *rhs)),
                    LogOp::Or => Ok(Value::Bool(*lhs || *rhs)),
                },
                _ => Err(EvalError::Unimplemented),
            },
        }
    }
}

#[cfg(test)]
mod test {
    use crate::interpreter::expression::{LogOp, Op};
    use crate::types::datatype::Value;

    use super::{EExpr, EOp, ExprTerm, FnName, RelOp, TExpr};

    #[test]
    pub fn expr_simple_1() {
        //TEST CASE
        //"1 > 2"

        let mut args = vec![Value::U64(1), Value::U64(2)];

        let expr = EExpr::Boolean(TExpr {
            operators: vec![Op {
                operands_ids: [0, 1],
                op_type: EOp::Rel(RelOp::Gt),
            }],
            terms: vec![ExprTerm::Arg(0), ExprTerm::Arg(1)],
        });

        let result = expr.eval(&mut args).unwrap();
        let expected_result = Value::Bool(false);

        assert_eq!(result, expected_result);
    }

    #[test]
    pub fn expr_fn_concat() {
        //TEST CASE
        //string = concat(["a", "b", "c"]) + "_group" //last one is binded

        let mut args = vec![
            Value::String("a".into()),
            Value::String("b".into()),
            Value::String("c".into()),
            Value::String("_group".into()),
        ];

        let expr = EExpr::Fn(FnName::Concat);

        let result = expr.eval(&mut args).unwrap();
        let expected_result = Value::String("abc_group".into());

        assert_eq!(result, expected_result);
    }
    #[test]
    pub fn expr_fn_concat_in_cond() {
        //TEST CASE
        //"abc_group" == concat(["a", "b", "c"]) + "_group"

        let mut args = vec![
            Value::String("a".into()),
            Value::String("b".into()),
            Value::String("c".into()),
            Value::String("_group".into()),
            Value::String("abc_group".into()),
        ];

        let expr = EExpr::Boolean(TExpr {
            operators: vec![Op {
                operands_ids: [0, 1],
                op_type: EOp::Rel(RelOp::Eqs),
            }],
            terms: vec![ExprTerm::Arg(4), ExprTerm::Fn(FnName::Concat, (0, 3))],
        });

        let result = expr.eval(&mut args).unwrap();
        let expected_result = Value::Bool(true);

        assert_eq!(result, expected_result);
    }

    #[test]
    pub fn expr_fn_concat_in_cond_or() {
        //TEST CASE
        //"abc_group" == concat(["a", "b", "c"]) + "_group" || 1 > 2

        let mut args = vec![
            Value::String("a".into()),
            Value::String("b".into()),
            Value::String("c".into()),
            Value::String("_group".into()),
            Value::String("abc_group".into()),
            Value::U64(1),
            Value::U64(2),
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
                ExprTerm::Fn(FnName::Concat, (0, 3)),
                ExprTerm::Arg(5),
                ExprTerm::Arg(6),
            ],
        });

        let result = expr.eval(&mut args).unwrap();
        let expected_result = Value::Bool(true);

        assert_eq!(result, expected_result);
    }
}
