#[allow(unreachable_patterns)]
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    json_types::U128,
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
    /// Argument at `ArgId` pos.
    Arg(ArgId),
    /// Function with defined interval (inclusive) for arguments.
    Fn(FnName, (ArgId, ArgId)),
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq))]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum FnName {
    /// Concat values to one string.
    /// Expect 0..N args.
    ///
    /// Return: `Value::String(result)`
    /// where result is from concated args.
    Concat,
    /// Return true if value is in the string.
    /// Expect 2 args where:
    /// - 0th arg is haystack
    /// - 1th arg is needle
    ///
    /// Return: `Value::Bool(result)`
    InString,
    /// Return true if value is in the array.
    /// Expect 2 args where:
    /// - 0th arg is haystack
    /// - 1th arg is needle
    ///
    /// Return: `Value::Bool(result)`
    InArray,
    /// Return value at index if exist, otherwise `Value::Null`.
    /// Expect 2 args where:
    /// - 0th arg is haystack
    /// - 1th arg is pos
    ///
    /// Return: `Value`
    ArrayAtIdx,
    /// Push value to the array, return array with the pushed value.
    /// Expect 2 args where:
    /// - 0th arg is array
    /// - 1th arg is value to be inserted
    ///
    /// Return: `Value`
    ArrayPush,
    /// Pops array's last value, return array without the last value.
    /// Expect 1 args where:
    /// - 0th arg is array
    ///
    /// Return: `Value`
    ArrayPop,
    /// Remove values from the array, return array without the values.
    /// Expect 2 args where:
    /// - 0th arg is array
    /// - 1th arg is the value to be removed
    ///
    /// Return: `Value`
    ArrayRemove,
    /// Return array length.
    /// Expect 1 args where:
    /// - 0th arg is array
    ///
    /// Return: `Value::U64`
    ArrayLen,
    /// Merge all values into one array, return merged array.
    /// Expect 0..N args.
    ///
    /// Return: `Value`
    ArrayMerge,
    /// Return true if value exists at the pos.
    /// Expect 1..N args where:
    /// - 0th arg is `Value::U64` defining position of tested value
    ///
    ValueExists,
    /// Return true if value at the pos is `Value::Null`.
    /// Expect 1..N args where:
    /// - 0th arg is `Value::U64` defining position of tested value
    ///
    /// Return: `Value::Bool`
    /// Return Error if no value is at the pos.
    ValueIsNull,
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
                    match val {
                        Value::String(ref v) => result.push_str(v),
                        Value::U64(ref v) => result.push_str(v.to_string().as_str()),
                        Value::U128(ref v) => result.push_str(v.0.to_string().as_str()),
                        _ => return Err(EvalError::InvalidDatatype),
                    };
                }
                Ok(Value::String(result))
            }
            FnName::InString => {
                if args.len() < 2 {
                    return Err(EvalError::InvalidArgCount(2));
                }
                match &args[0] {
                    Value::String(v) => {
                        let needle = args[1].try_into_str()?;
                        let result = v.contains(needle);
                        Ok(Value::Bool(result))
                    }
                    _ => Err(EvalError::InvalidDatatype),
                }
            }
            FnName::InArray => {
                if args.len() < 2 {
                    return Err(EvalError::InvalidArgCount(2));
                }
                match &args[0] {
                    Value::VecString(vec) => {
                        let needle = args[1].try_into_str()?;
                        let result = vec.iter().any(|v| v == needle);
                        Ok(Value::Bool(result))
                    }
                    Value::VecU64(vec) => {
                        let needle = args[1].try_into_u64()?;
                        let result = vec.iter().any(|v| *v == needle);
                        Ok(Value::Bool(result))
                    }
                    Value::VecU128(vec) => {
                        let needle = args[1].try_into_u128()?;
                        let result = vec.iter().any(|v| v.0 == needle);
                        Ok(Value::Bool(result))
                    }
                    _ => Err(EvalError::InvalidDatatype),
                }
            }
            FnName::ArrayAtIdx => {
                if args.len() < 2 {
                    return Err(EvalError::InvalidArgCount(2));
                }
                match &args[0] {
                    Value::VecString(vec) => {
                        let index = args[1].try_into_u64()?;
                        let value = match vec.get(index as usize) {
                            Some(v) => Value::String(v.clone()),
                            None => Value::Null,
                        };
                        Ok(value)
                    }
                    Value::VecU64(vec) => {
                        let index = args[1].try_into_u64()?;
                        let value = match vec.get(index as usize) {
                            Some(v) => Value::U64(*v),
                            None => Value::Null,
                        };
                        Ok(value)
                    }
                    Value::VecU128(vec) => {
                        let index = args[1].try_into_u64()?;
                        let value = match vec.get(index as usize) {
                            Some(v) => Value::U128(*v),
                            None => Value::Null,
                        };
                        Ok(value)
                    }
                    _ => Err(EvalError::InvalidDatatype),
                }
            }
            FnName::ArrayPush => {
                if args.len() < 2 {
                    return Err(EvalError::InvalidArgCount(2));
                }
                match &args[0] {
                    Value::VecString(vec) => {
                        let target_value = args[1].clone().try_into_string()?;
                        let mut new_vec = vec.to_owned();
                        new_vec.push(target_value);
                        Ok(Value::VecString(new_vec))
                    }
                    Value::VecU64(vec) => {
                        let target_value = args[1].try_into_u64()?;
                        let mut new_vec = vec.to_owned();
                        new_vec.push(target_value);
                        Ok(Value::VecU64(new_vec))
                    }
                    Value::VecU128(vec) => {
                        let target_value = args[1].try_into_u128()?;
                        let mut new_vec = vec.to_owned();
                        new_vec.push(target_value.into());
                        Ok(Value::VecU128(new_vec))
                    }
                    _ => Err(EvalError::InvalidDatatype),
                }
            }
            FnName::ArrayPop => {
                if args.len() < 1 {
                    return Err(EvalError::InvalidArgCount(1));
                }
                match &args[0] {
                    Value::VecString(vec) => {
                        let mut new_vec = vec.to_owned();
                        new_vec.pop();
                        Ok(Value::VecString(new_vec))
                    }
                    Value::VecU64(vec) => {
                        let mut new_vec = vec.to_owned();
                        new_vec.pop();
                        Ok(Value::VecU64(new_vec))
                    }
                    Value::VecU128(vec) => {
                        let mut new_vec = vec.to_owned();
                        new_vec.pop();
                        Ok(Value::VecU128(new_vec))
                    }
                    _ => Err(EvalError::InvalidDatatype),
                }
            }
            FnName::ArrayRemove => {
                if args.len() < 1 {
                    return Err(EvalError::InvalidArgCount(1));
                }
                match &args[0] {
                    Value::VecU64(v) => {
                        let needle = args[1].try_into_u64()?;
                        let v = v
                            .iter()
                            .filter_map(|v| {
                                if *v != needle {
                                    Some(v.to_owned())
                                } else {
                                    None
                                }
                            })
                            .collect();
                        Ok(Value::VecU64(v))
                    }
                    Value::VecU128(v) => {
                        let needle = args[1].try_into_u128()?;
                        let v = v
                            .iter()
                            .filter_map(|v| {
                                if v.0 != needle {
                                    Some(v.to_owned())
                                } else {
                                    None
                                }
                            })
                            .collect();
                        Ok(Value::VecU128(v))
                    }
                    Value::VecString(v) => {
                        let needle = args[1].try_into_str()?;
                        let v = v
                            .iter()
                            .filter_map(|v| {
                                if *v != needle {
                                    Some(v.to_owned())
                                } else {
                                    None
                                }
                            })
                            .collect();
                        Ok(Value::VecString(v))
                    }
                    _ => Err(EvalError::Unimplemented),
                }
            }
            FnName::ArrayMerge => {
                if args.len() < 1 {
                    return Err(EvalError::InvalidArgCount(1));
                }
                match &args[0] {
                    Value::String(_) => {
                        let mut result: Vec<String> = Vec::with_capacity(32);
                        for arg in args.iter() {
                            let v = arg.clone().try_into_string()?;
                            result.push(v);
                        }
                        Ok(Value::VecString(result))
                    },
                    Value::U64(_) => {
                        let mut result: Vec<u64> = Vec::with_capacity(32);
                        for arg in args.iter() {
                            let v = arg.try_into_u64()?;
                            result.push(v);
                        }
                        Ok(Value::VecU64(result))
                    },
                    Value::U128(_) => {
                        let mut result: Vec<U128> = Vec::with_capacity(32);
                        for arg in args.iter() {
                            let v = arg.try_into_u128()?;
                            result.push(v.into());
                        }
                        Ok(Value::VecU128(result))
                    },
                    Value::VecString(_) => {
                        let mut result: Vec<String> = Vec::with_capacity(32);
                        for arg in args.iter() {
                            let mut v = arg.clone().try_into_vec_string()?;
                            result.append(&mut v);
                        }
                        Ok(Value::VecString(result))
                    }
                    Value::VecU128(_) => {
                        let mut result: Vec<U128> = Vec::with_capacity(32);
                        for arg in args.iter() {
                            let mut v = arg.clone().try_into_vec_u128()?;
                            result.append(&mut v);
                        }
                        Ok(Value::VecU128(result))
                    }
                    Value::VecU64(_) => {
                        let mut result: Vec<u64> = Vec::with_capacity(32);
                        for arg in args.iter() {
                            let mut v = arg.clone().try_into_vec_u64()?;
                            result.append(&mut v);
                        }
                        Ok(Value::VecU64(result))
                    }
                    _ => Err(EvalError::InvalidDatatype),
                }
            }
            FnName::ArrayLen => {
                if args.len() < 1 {
                    return Err(EvalError::InvalidArgCount(1));
                }
                match &args[0] {
                    Value::VecBool(v) => Ok(Value::U64(v.len() as u64)),
                    Value::VecU64(v) => Ok(Value::U64(v.len() as u64)),
                    Value::VecU128(v) => Ok(Value::U64(v.len() as u64)),
                    Value::VecString(v) => Ok(Value::U64(v.len() as u64)),
                    _ => Err(EvalError::InvalidDatatype),
                }
            }
            FnName::ValueExists => {
                if args.len() < 1 {
                    return Err(EvalError::InvalidArgCount(1));
                }
                let pos = args[0].try_into_u64()?;
                let result = args.get(pos as usize).is_some();
                Ok(Value::Bool(result))
            }
            FnName::ValueIsNull => {
                if args.len() < 1 {
                    return Err(EvalError::InvalidArgCount(1));
                }
                let pos = args[0].try_into_u64()?;
                match args.get(pos as usize) {
                    Some(v) => Ok(Value::Bool(v.is_null())),
                    None => Err(EvalError::MissingArg(pos)),
                }
            }
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
                _ => Err(EvalError::InvalidOperands("aritmetic operation".into())),
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
                _ => Err(EvalError::InvalidDatatype),
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
    pub fn expr_gt() {
        //TEST CASE
        //"1 > 2"
        let args = vec![Value::U64(1), Value::U64(2)];
        let expr = EExpr::Boolean(TExpr {
            operators: vec![Op {
                operands_ids: [0, 1],
                op_type: EOp::Rel(RelOp::Gt),
            }],
            terms: vec![ExprTerm::Arg(0), ExprTerm::Arg(1)],
        });
        let result = expr.eval(&args).unwrap();
        let expected_result = Value::Bool(false);
        assert_eq!(result, expected_result);
    }
    #[test]
    pub fn expr_fn_concat_in_cond() {
        //TEST CASE
        //"abc_group" == concat(["a", "b", "c"]) + "_group"
        let args = vec![
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
        let result = expr.eval(&args).unwrap();
        let expected_result = Value::Bool(true);
        assert_eq!(result, expected_result);
    }

    #[test]
    pub fn expr_fn_concat_in_cond_or() {
        //TEST CASE
        //"abc_group" == concat(["a", "b", "c"]) + "_group" || 1 > 2
        let args = vec![
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
        let result = expr.eval(&args).unwrap();
        let expected_result = Value::Bool(true);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn expr_fn_concat() {
        let expr = EExpr::Fn(FnName::Concat);
        let args = vec![
            Value::String("a".into()),
            Value::String("b".into()),
            Value::String("c".into()),
            Value::String("_group".into()),
        ];
        let result = expr.eval(&args).unwrap();
        let expected_result = Value::String("abc_group".into());
        assert_eq!(result, expected_result);
    }
    #[test]
    fn expr_fn_in_string() {
        let expr = EExpr::Fn(FnName::InString);
        let haystack = Value::String("abc_group".into());
        let needle = Value::String("_gr".into());
        let args = vec![haystack, needle];
        let result = expr.eval(&args).unwrap();
        assert_eq!(result, Value::Bool(true));
    }
    #[test]
    fn expr_fn_in_array() {
        let expr = EExpr::Fn(FnName::InArray);
        let haystack = Value::VecString(vec!["abc".into(), "group".into(), "abc_group".into()]);
        let needle = Value::String("abc_group".into());
        let args = vec![haystack.clone(), needle];
        let result = expr.eval(&args).unwrap();
        assert_eq!(result, Value::Bool(true));
        let needle = Value::String("ab".into());
        let args = vec![haystack.clone(), needle];
        let result = expr.eval(&args).unwrap();
        assert_eq!(result, Value::Bool(false));
    }
    #[test]
    fn expr_fn_array_at_index() {
        let expr = EExpr::Fn(FnName::ArrayAtIdx);
        let haystack = Value::VecString(vec!["abc".into(), "group".into(), "abc_group".into()]);
        let pos = Value::U64(0);
        let args = vec![haystack.clone(), pos];
        let result = expr.eval(&args).unwrap();
        assert_eq!(result, Value::String("abc".into()));
        let pos = Value::U64(3);
        let args = vec![haystack.clone(), pos];
        let result = expr.eval(&args).unwrap();
        assert_eq!(result, Value::Null);
    }
    #[test]
    fn expr_fn_array_push() {
        let expr = EExpr::Fn(FnName::ArrayPush);
        let previous = Value::VecString(vec!["abc".into(), "group".into(), "abc_group".into()]);
        let args = vec![previous, Value::String("ab".into())];
        let result = expr.eval(&args).unwrap();
        let expected = Value::VecString(vec![
            "abc".into(),
            "group".into(),
            "abc_group".into(),
            "ab".into(),
        ]);
        assert_eq!(result, expected);
    }
    #[test]
    fn expr_fn_array_pop() {
        let expr = EExpr::Fn(FnName::ArrayPop);
        let previous = Value::VecString(vec![
            "abc".into(),
            "group".into(),
            "abc_group".into(),
            "ab".into(),
        ]);
        let args = vec![previous];
        let result = expr.eval(&args).unwrap();
        let expected = Value::VecString(vec!["abc".into(), "group".into(), "abc_group".into()]);
        assert_eq!(result, expected);
    }
    #[test]
    fn expr_fn_array_remove() {
        let expr = EExpr::Fn(FnName::ArrayRemove);
        let haystack = Value::VecString(vec![
            "abc".into(),
            "group".into(),
            "abc".into(),
            "abc_group".into(),
        ]);
        let needle = Value::String("abc".into());
        let args = vec![haystack, needle];
        let result = expr.eval(&args).unwrap();
        let expected = Value::VecString(vec!["group".into(), "abc_group".into()]);
        assert_eq!(result, expected);
    }
    #[test]
    fn expr_fn_array_len() {
        let expr = EExpr::Fn(FnName::ArrayLen);
        let value = Value::VecString(vec!["abc".into(), "group".into(), "abc_group".into()]);
        let args = vec![value];
        let result = expr.eval(&args).unwrap();
        assert_eq!(result, Value::U64(3));
    }
    #[test]
    fn expr_fn_array_merge() {
        let expr = EExpr::Fn(FnName::ArrayMerge);
        let vec_1 = Value::VecString(vec!["1abc".into(), "group".into(), "abc_group".into()]);
        let vec_2 = Value::VecString(vec!["2abc".into(), "group".into(), "abc_group".into()]);
        let vec_3 = Value::VecString(vec!["3abc".into(), "group".into(), "abc_group".into()]);
        let args = vec![vec_1, vec_2, vec_3];
        let result = expr.eval(&args).unwrap();
        let expected = Value::VecString(vec![
            "1abc".into(),
            "group".into(),
            "abc_group".into(),
            "2abc".into(),
            "group".into(),
            "abc_group".into(),
            "3abc".into(),
            "group".into(),
            "abc_group".into(),
        ]);
        assert_eq!(result, expected);
        let val_1 = Value::String("4abc".into());
        let val_2 = Value::String("group".into());
        let val_3 = Value::String("abc_group".into());
        let args = vec![val_1, val_2, val_3];
        let result = expr.eval(&args).unwrap();
        let expected = Value::VecString(vec![
            "4abc".into(),
            "group".into(),
            "abc_group".into(),
        ]);
        assert_eq!(result, expected);
    }
    #[test]
    fn expr_fn_value_exists() {
        let expr = EExpr::Fn(FnName::ValueExists);
        let value = Value::VecString(vec!["abc".into(), "group".into(), "abc_group".into()]);
        let pos = Value::U64(1);
        let args = vec![pos, value];
        let result = expr.eval(&args).unwrap();
        assert_eq!(result, Value::Bool(true));
        let pos = Value::U64(1);
        let args = vec![pos];
        let result = expr.eval(&args).unwrap();
        assert_eq!(result, Value::Bool(false));
    }
    #[test]
    fn expr_fn_value_is_null() {
        let expr = EExpr::Fn(FnName::ValueIsNull);
        let value = Value::VecString(vec!["abc".into(), "group".into(), "abc_group".into()]);
        let pos = Value::U64(1);
        let args = vec![pos, value];
        let result = expr.eval(&args).unwrap();
        assert_eq!(result, Value::Bool(false));
        let pos = Value::U64(1);
        let args = vec![pos, Value::Null];
        let result = expr.eval(&args).unwrap();
        assert_eq!(result, Value::Bool(true));
    }
}
