use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
};

use crate::types::DataType;

type ArgId = u8;

// TODO error trait
pub const ERR_OPERANDS: &str = "Incompatible operands";
pub const ERR_OPERATION: &str = "Invalid operation";
pub const ERR_DATATYPE: &str = "Invalid datatype";

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
    ArrayLen,
    ToArray,
    ValueExists,
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
            panic!("{}", ERR_DATATYPE);
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
                        _ => panic!("{}", ERR_DATATYPE),
                    };
                }
                DataType::String(result)
            }
            FnName::ToArray => match &args[0] {
                DataType::String(_) => {
                    let mut result = Vec::with_capacity(args.len());
                    for arg in args.iter() {
                        result.push(arg.clone().try_into_string().expect(ERR_DATATYPE));
                    }
                    DataType::VecString(result)
                }
                _ => panic!("{}", ERR_OPERATION),
            },
            FnName::ValueExists => match &args.get(0) {
                Some(_) => DataType::Bool(true),
                None => DataType::Bool(false),
            },
            _ => panic!("{}", ERR_OPERATION),
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
            EOp::Ari(o) => match (arg1, arg2) {
                (DataType::U8(lhs), DataType::U8(rhs)) => {
                    let result = match o {
                        AriOp::Add => lhs + rhs,
                        AriOp::Subtract => lhs - rhs,
                        AriOp::Multiply => lhs * rhs,
                        AriOp::Divide => lhs / rhs,
                        AriOp::Modulo => lhs % rhs,
                    };

                    DataType::U8(result)
                }
                (DataType::U128(lhs), DataType::U128(rhs)) => {
                    let result = match o {
                        AriOp::Add => lhs.0 + rhs.0,
                        AriOp::Subtract => lhs.0 - rhs.0,
                        AriOp::Multiply => lhs.0 * rhs.0,
                        AriOp::Divide => lhs.0 / rhs.0,
                        AriOp::Modulo => lhs.0 % rhs.0,
                    };

                    DataType::U128(result.into())
                }
                _ => panic!("Invalid operands for aritmetic operation"),
            },
            EOp::Rel(o) => match (arg1, arg2) {
                (DataType::Bool(lhs), DataType::Bool(rhs)) => match o {
                    RelOp::Eqs => DataType::Bool(*lhs == *rhs),
                    RelOp::NEqs => DataType::Bool(*lhs != *rhs),
                    _ => panic!("{}", ERR_OPERATION),
                },
                (DataType::U8(lhs), DataType::U8(rhs)) => match o {
                    RelOp::Eqs => DataType::Bool(lhs == rhs),
                    RelOp::NEqs => DataType::Bool(lhs != rhs),
                    RelOp::Gt => DataType::Bool(lhs > rhs),
                    RelOp::Lt => DataType::Bool(lhs < rhs),
                    RelOp::GtE => DataType::Bool(lhs >= rhs),
                    RelOp::LtE => DataType::Bool(lhs <= rhs),
                },
                (DataType::U16(lhs), DataType::U16(rhs)) => match o {
                    RelOp::Eqs => DataType::Bool(lhs == rhs),
                    RelOp::NEqs => DataType::Bool(lhs != rhs),
                    RelOp::Gt => DataType::Bool(lhs > rhs),
                    RelOp::Lt => DataType::Bool(lhs < rhs),
                    RelOp::GtE => DataType::Bool(lhs >= rhs),
                    RelOp::LtE => DataType::Bool(lhs <= rhs),
                },
                (DataType::U128(lhs), DataType::U128(rhs)) => match o {
                    RelOp::Eqs => DataType::Bool(lhs == rhs),
                    RelOp::NEqs => DataType::Bool(lhs != rhs),
                    RelOp::Gt => DataType::Bool(lhs.0 > rhs.0),
                    RelOp::Lt => DataType::Bool(lhs.0 < rhs.0),
                    RelOp::GtE => DataType::Bool(lhs.0 >= rhs.0),
                    RelOp::LtE => DataType::Bool(lhs.0 <= rhs.0),
                },
                (DataType::String(lhs), DataType::String(rhs)) => match o {
                    RelOp::Eqs => DataType::Bool(*lhs == *rhs),
                    RelOp::NEqs => DataType::Bool(*lhs != *rhs),
                    RelOp::Gt => DataType::Bool(*lhs > *rhs),
                    RelOp::Lt => DataType::Bool(*lhs < *rhs),
                    RelOp::GtE => DataType::Bool(*lhs >= *rhs),
                    RelOp::LtE => DataType::Bool(*lhs <= *rhs),
                },
                _ => panic!("{}", ERR_OPERANDS),
            },
            EOp::Log(o) => match (arg1, arg2) {
                (DataType::Bool(lhs), DataType::Bool(rhs)) => match o {
                    LogOp::And => DataType::Bool(*lhs && *rhs),
                    LogOp::Or => DataType::Bool(*lhs || *rhs),
                },
                _ => panic!("{}", ERR_OPERATION),
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
