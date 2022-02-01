use std::collections::HashMap;

/*
Entities:
Expr -> Term | Operator
Term -> Factor | Operator
Factor -> Constant | Variable


Wanted:

Expression is recursive structure

Evaluating expressions with basic data types: string, bool, integer, arrays and their operations

Integer operations: Add/Subtract/Multiply/Divide
String: in string, concat
Boolean: equals
Array: in array, merge ??

Expression must have its return type - based on operands

Example:
register b = 4;
if (register a == 10) {
    register b = 3 * (2 * a + b + 1);
} else {
    register c = 2 * a ;
}
register d = 2 * a;

BTW we assume registers are just variables and we can get their values
let a, b, c: integers;

1. check variable a value = evaluate BinaryExpression
_____________
2. true path: evaluate AritmeticExpression (3 * Aritmetic expression (2 * a + b + 1))
3. assign result to b
____________
2. false path: evaluate AritmeticExpression 2 * a;
3. assign result to c

4. evaluate AritmeticExpression 2 * a
assign result to C

----------------
Program has:
variables
conditions
statements
expressions
output
_______
let c = registr; //s0
let b = input; //s1
let a = 10; //s2
if (a > input) { //s3 //c1
    c = b * 2 //s4 c1 = true
} else {
    b = c * a //s5 c1 = false
}
d = a + b + c s6

statements[0, 1, 2,3c, 4, 5, 6]
conditions[c1]
*/

use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
};

use crate::storage::DataType as DT;

type ExprNode = Box<TExpr>;
type ArgId = u8;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum ExprTerm {
    Value(DT),
    Arg(ArgId),
    FnCall(FnName, (u8, u8)),
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
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

/*
 concat([a,b,c]) == "abc" || 1 > 2

    operands: [eqs(0,1), eqs(2,3), logic(0,1)]
    terms: [concat([0,0]), arg(1), arg(2) , arg(3)],
    res: [term[0]]

*/

// Recursive structure does not work with deserializer
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct TExpr {
    pub operators: Vec<Op>,
    pub terms: Vec<ExprTerm>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Op {
    pub operands_ids: [u8; 2],
    pub op_type: Operator,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum EExpr {
    Aritmetic(TExpr),
    Boolean(TExpr),
    String(TExpr),
    Fn(FnName),
}

impl EExpr {
    pub fn eval(&self, args: &mut Vec<DT>) -> DT {
        match self {
            EExpr::Aritmetic(e) | EExpr::Boolean(e) | EExpr::String(e) => {
                self.eval_expr(e, args).unwrap()
            }
            EExpr::Fn(fn_name) => self.eval_fn(fn_name, args.as_slice()),
        }
    }

    pub fn eval_expr(&self, expr: &TExpr, args: &[DT]) -> Result<DT, String> {
        let mut results = Vec::with_capacity(expr.terms.len());
        for op in expr.operators.iter() {

            let temp_res = match &op.op_type {
                Operator::Logic(_) => {
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

        //Err("undefined".into())
    }

    fn eval_fn(&self, fn_name: &FnName, args: &[DT]) -> DT {
        match fn_name {
            FnName::Concat => {
                let mut result = String::with_capacity(64);

                for i in 0..args.len() {
                    // cannot be None coz we iterate by the array
                    match args.get(i).unwrap() {
                        DT::String(ref v) => result.push_str(v),
                        _ => panic!("{}", "Expected DT::VecString"),
                    };
                }
                DT::String(result)
            }
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

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum AritmeticOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum RelationalOperation {
    Eqs,
    NEqs,
    Gt,
    Lt,
    GtE,
    LtE,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum LogicOperation {
    And,
    Or,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum Operator {
    Aritmetic(AritmeticOperation),
    Logic(LogicOperation),
    Relational(RelationalOperation),
}

impl Operator {
    pub fn operate(&self, arg1: &DT, arg2: &DT) -> DT {
        match self {
            Operator::Aritmetic(o) => {
                let (lhs, rhs) = match (arg1, arg2) {
                    (DT::U8(lhs), DT::U8(rhs)) => (*lhs, *rhs),
                    _ => panic!("Invalid operands for aritmetic operation"),
                };

                let result = match o {
                    AritmeticOperation::Add => lhs + rhs,
                    AritmeticOperation::Subtract => lhs - rhs,
                    AritmeticOperation::Multiply => lhs * rhs,
                    AritmeticOperation::Divide => lhs / rhs,
                    AritmeticOperation::Modulo => lhs % rhs,
                };

                DT::U8(result)
            }
            Operator::Aritmetic(o) => {
                let (lhs, rhs) = match (arg1, arg2) {
                    (DT::U128(lhs), DT::U128(rhs)) => (*lhs, *rhs),
                    _ => panic!("Invalid operands for aritmetic operation"),
                };

                let result = match o {
                    AritmeticOperation::Add => lhs + rhs,
                    AritmeticOperation::Subtract => lhs - rhs,
                    AritmeticOperation::Multiply => lhs * rhs,
                    AritmeticOperation::Divide => lhs / rhs,
                    AritmeticOperation::Modulo => lhs % rhs,
                };

                DT::U128(result)
            }
            Operator::Relational(o) => match (arg1, arg2) {
                (DT::Bool(lhs), DT::Bool(rhs)) => match o {
                    RelationalOperation::Eqs => DT::Bool(*lhs == *rhs),
                    RelationalOperation::NEqs => DT::Bool(*lhs != *rhs),
                    _ => panic!("Invalid operation"),
                },
                (DT::U8(lhs), DT::U8(rhs)) => match o {
                    RelationalOperation::Eqs => DT::Bool(lhs == rhs),
                    RelationalOperation::NEqs => DT::Bool(lhs != rhs),
                    RelationalOperation::Gt => DT::Bool(lhs > rhs),
                    RelationalOperation::Lt => DT::Bool(lhs < rhs),
                    RelationalOperation::GtE => DT::Bool(lhs >= rhs),
                    RelationalOperation::LtE => DT::Bool(lhs <= rhs),
                    _ => panic!("Invalid operands"),
                },
                (DT::String(lhs), DT::String(rhs)) => match o {
                    RelationalOperation::Eqs => DT::Bool(*lhs == *rhs),
                    RelationalOperation::NEqs => DT::Bool(*lhs != *rhs),
                    RelationalOperation::Gt => DT::Bool(*lhs > *rhs),
                    RelationalOperation::Lt => DT::Bool(*lhs < *rhs),
                    RelationalOperation::GtE => DT::Bool(*lhs >= *rhs),
                    RelationalOperation::LtE => DT::Bool(*lhs <= *rhs),
                    _ => panic!("Invalid operands"),
                },
                // TODO: which operations
                //(DT::VecString(lhs), DT::VecString(rhs)) => match o {
                //    _ => panic!("Invalid operation"),
                //},
                //(DT::ArrInteger(lhs), DT::ArrInteger(rhs)) => match o {
                //    _ => panic!("Invalid operation"),
                //},
                _ => panic!("Invalid operand types for this operation"),
            },
            Operator::Logic(o) => match (arg1, arg2) {
                (DT::Bool(lhs), DT::Bool(rhs)) => match o {
                    LogicOperation::And => DT::Bool(*lhs && *rhs),
                    LogicOperation::Or => DT::Bool(*lhs || *rhs),
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
        expression::{LogicOperation, Op},
        storage::DataType,
    };

    use super::{EExpr, ExprTerm, FnName, Operator, RelationalOperation, TExpr};

    #[test]
    pub fn expr_simple_1() {
        //TEST CASE
        //"1 > 2"

        let mut args = vec![DataType::U8(1), DataType::U8(2)];

        let expr = EExpr::Boolean(TExpr {
            operators: vec![Op {
                operands_ids: [0, 1],
                op_type: Operator::Relational(RelationalOperation::Gt),
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
                op_type: Operator::Relational(RelationalOperation::Eqs),
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
                    op_type: Operator::Relational(RelationalOperation::Eqs),
                },
                Op {
                    operands_ids: [2, 3],
                    op_type: Operator::Relational(RelationalOperation::Gt),
                },
                Op {
                    operands_ids: [0, 1],
                    op_type: Operator::Logic(LogicOperation::Or),
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

// ---------------- OLD BELLOW - REMOVE

/*

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
enum Value {
    Null,
    String(String),
    Integer(u128),
    Boolean(bool),
    ArrString(Vec<String>),
    ArrInteger(Vec<u128>),
}

struct Statement {
    var_name: String,
    value: StatementVal,
}

enum StatementVal {
    Expr(Expr),
    Const(Value),
    Variable(String),
    Register(String),
    Input,
    FnCall(Fn, Vec<FnArg>),
}

enum Instruction {
    Statement(Statement),
    Condition(usize),
}

struct Condition {
    expr: ConditionExpr,
    true_path: Vec<usize>,
    false_path: Vec<usize>,
    ret: usize,
}

enum ConditionExpr {
    Expr(Expr),
    FnCall(Fn, Vec<FnArg>),
}

struct Program {
    //vmap: VarMap,
    instructions: Vec<Instruction>,
    conditions: Vec<Condition>,
    //counter: usize,
    input: Value,
    output: Value,
}

struct Interpreter {
    vmap: VarMap,
}

enum InstructionResult {
    Ok,
    Jump(usize, bool, usize),
    JumpEnd(usize),
}

type Counter = usize;
type InstructionIdx = usize;
type Path = bool;
type CondInstrFlags = Option<(InstructionIdx, Path, Counter)>;

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            vmap: VarMap::new(),
        }
    }

    /* // Saves requested variables into registers
    pub fn store_registers<T: AsRef<str>>(
        &self,
        register_api: &mut Register,
        variables: &[T],
        registers: &[T],
    ) -> Result<(), RuntimeErr> {
        for i in 0..variables.len() {
            register_api.store(
                registers[i].as_ref(),
                self.vmap
                    .get(variables[i].as_ref())
                    .ok_or(RuntimeErr::UndefinedVariable)?,
            );
        }

        Ok(())
    } */

    pub fn eval(&mut self, program: &mut Program, register_api: &mut VarMap) -> Value {
        let mut counter = 0;
        let mut cond: CondInstrFlags = None;
        while counter < program.instructions.len() {
            dbg!(counter);
            let instruction = program.instructions.get(counter).unwrap();
            match self.execute_instruction(
                instruction,
                program.conditions.as_slice(),
                register_api,
                counter,
                &cond,
            ) {
                // set values where we are in cond. jump
                InstructionResult::Ok => {
                    counter += 1;
                }
                InstructionResult::Jump(cond_idx, path, instr_idx) => {
                    cond = Some((cond_idx, path, instr_idx));
                    counter = instr_idx;
                }
                InstructionResult::JumpEnd(ret_instr_idx) => {
                    cond = None;
                    counter = ret_instr_idx;
                }
            }
        }

        program.output = self.vmap.get("output").unwrap_or(Value::Null);
        program.output.clone()
    }

    fn execute_instruction(
        &mut self,
        instruction: &Instruction,
        conditions: &[Condition],
        register_api: &mut VarMap,
        counter: usize,
        cond: &CondInstrFlags,
    ) -> InstructionResult {
        match instruction {
            Instruction::Statement(s) => {
                let value: Value = match &s.value {
                    StatementVal::Expr(expr) => match expr {
                        Expr::Aritmetic(e) => eval_arit_expr(&e, &mut self.vmap),
                        Expr::Boolean(e) => eval_boolean_expr(&e, &mut self.vmap),
                    },
                    StatementVal::Const(c) => c.clone(),
                    StatementVal::Variable(s) => self.vmap.get(&s).expect("Undefied variable"),
                    StatementVal::Register(r) => register_api.get(&r).expect("Undefined register"),
                    StatementVal::Input => self.vmap.get("input").unwrap(),
                    StatementVal::FnCall(func, args) => {
                        eval_fn(func, args, &self.vmap).expect("Fn eval error")
                    }
                };

                self.vmap.set(&s.var_name, &value);

                dbg!(cond);

                // Check if we execute conditional jump instructions
                if let Some((cond_idx, path, instr_idx)) = cond {
                    //panic!("cond_idx: {}", cond_idx);
                    // check if we reached end of conditional instruction

                    let condition = conditions.get(*cond_idx).expect("Undefined condition");

                    let cond_instr_path = match *path {
                        true => &condition.true_path,
                        false => &condition.false_path,
                    };
                    dbg!(*cond_instr_path
                        .last()
                        .expect("Undefined last instruction in cond"));

                    if counter
                        == *cond_instr_path
                            .last()
                            .expect("Undefined last instruction in cond")
                    {
                        InstructionResult::JumpEnd(condition.ret)
                    } else {
                        InstructionResult::Ok
                    }
                } else {
                    InstructionResult::Ok
                }
            }
            Instruction::Condition(n) => {
                let cond = conditions.get(*n).expect("Undefined condition");

                let v: bool = match &cond.expr {
                    ConditionExpr::FnCall(func, args) => {
                        match eval_fn(func, args, &self.vmap).unwrap() {
                            Value::Boolean(val) => val,
                            _ => panic!("invalid value datatype"),
                        }
                    }
                    ConditionExpr::Expr(expr) => match expr {
                        Expr::Boolean(expr) => match eval_boolean_expr(expr, &mut self.vmap) {
                            Value::Boolean(val) => val,
                            _ => panic!("invalid value datatype"),
                        },
                        _ => panic!("Invalid expression type in condition"),
                    },
                };

                // Set conditional jump
                match v {
                    true => InstructionResult::Jump(*n, v, *cond.true_path.get(0).unwrap()),
                    false => InstructionResult::Jump(*n, v, *cond.false_path.get(0).unwrap()),
                }
            }
        }
    }
}

#[derive(Debug)]
enum RuntimeErr {
    InvalidFunctionArgument,
    InvalidFunctionArgCount,
    InvalidOperand,
    UndefinedOperation,
    UndefinedVariable,
    UnexpectedDataType,
}

enum DataType {
    String,
    Integer,
    Boolean,
    Array,
}

enum FnName {
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

struct Fn {
    name: FnName,
    ret_type: DataType,
}

fn validate_type(datatype: &DataType, value: &Value) -> bool {
    match (datatype, value) {
        (DataType::String, Value::String(_)) => true,
        (DataType::Integer, Value::Integer(_)) => true,
        (DataType::Boolean, Value::Boolean(_)) => true,
        (DataType::Array, Value::ArrString(_)) => true,
        (DataType::Array, Value::ArrInteger(_)) => true,
        _ => false,
    }
}

enum AritmeticOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

enum RelationalOperation {
    Eqs,
    NEqs,
    Gt,
    Lt,
    GtE,
    LtE,
}

enum LogicOperation {
    And,
    Or,
}

enum Operator {
    Aritmetic(AritmeticOperation),
    Logic(LogicOperation),
    Relational(RelationalOperation),
}

impl Operator {
    pub fn operate(&self, arg1: &Value, arg2: &Value) -> Value {
        match self {
            Operator::Aritmetic(o) => {
                let (lhs, rhs) = match (arg1, arg2) {
                    (Value::Integer(lhs), Value::Integer(rhs)) => (*lhs, *rhs),
                    _ => panic!("Invalid operands for aritmetic operation"),
                };

                let result = match o {
                    AritmeticOperation::Add => lhs + rhs,
                    AritmeticOperation::Subtract => lhs - rhs,
                    AritmeticOperation::Multiply => lhs * rhs,
                    AritmeticOperation::Divide => lhs / rhs,
                    AritmeticOperation::Modulo => lhs % rhs,
                };

                Value::Integer(result)
            }
            Operator::Relational(o) => match (arg1, arg2) {
                (Value::Boolean(lhs), Value::Boolean(rhs)) => match o {
                    RelationalOperation::Eqs => Value::Boolean(*lhs == *rhs),
                    RelationalOperation::NEqs => Value::Boolean(*lhs != *rhs),
                    _ => panic!("Invalid operation"),
                },
                (Value::Integer(lhs), Value::Integer(rhs)) => match o {
                    RelationalOperation::Eqs => Value::Boolean(lhs == rhs),
                    RelationalOperation::NEqs => Value::Boolean(lhs != rhs),
                    RelationalOperation::Gt => Value::Boolean(lhs > rhs),
                    RelationalOperation::Lt => Value::Boolean(lhs < rhs),
                    RelationalOperation::GtE => Value::Boolean(lhs >= rhs),
                    RelationalOperation::LtE => Value::Boolean(lhs <= rhs),
                    _ => panic!("Invalid operands"),
                },
                (Value::String(lhs), Value::String(rhs)) => match o {
                    RelationalOperation::Eqs => Value::Boolean(*lhs == *rhs),
                    RelationalOperation::NEqs => Value::Boolean(*lhs != *rhs),
                    RelationalOperation::Gt => Value::Boolean(*lhs > *rhs),
                    RelationalOperation::Lt => Value::Boolean(*lhs < *rhs),
                    RelationalOperation::GtE => Value::Boolean(*lhs >= *rhs),
                    RelationalOperation::LtE => Value::Boolean(*lhs <= *rhs),
                    _ => panic!("Invalid operands"),
                },
                // TODO: which operations
                (Value::ArrString(lhs), Value::ArrString(rhs)) => match o {
                    _ => panic!("Invalid operation"),
                },
                (Value::ArrInteger(lhs), Value::ArrInteger(rhs)) => match o {
                    _ => panic!("Invalid operation"),
                },
                _ => panic!("Invalid operand tyes for this operation"),
            },
            Operator::Logic(o) => match (arg1, arg2) {
                (Value::Boolean(lhs), Value::Boolean(rhs)) => match o {
                    LogicOperation::And => Value::Boolean(*lhs && *rhs),
                    LogicOperation::Or => Value::Boolean(*lhs || *rhs),
                    _ => panic!("Invalid operation"),
                },
                _ => panic!("Invalid operand tyes for this operation"),
            },
        }
    }
}

enum Expr {
    Aritmetic(Expression),
    Boolean(Expression),
}

enum FnArg {
    Value(Value),
    Variable(String),
}

enum ExprTerm {
    Node(ExprNode),
    Value(Value),
    Variable(String),
    FnCall(Fn, Vec<FnArg>),
}
struct Expression {
    op: Operator,
    lhs: ExprTerm,
    rhs: ExprTerm,
}

#[derive(Debug)]
struct VarMap(HashMap<String, Value>);

impl VarMap {
    pub fn new() -> Self {
        VarMap(HashMap::new())
    }

    pub fn get(&self, var_name: &str) -> Option<Value> {
        self.0.get(var_name).map(|v| v.clone())
    }

    pub fn set(&mut self, var_name: &str, value: &Value) -> Option<Value> {
        self.0.insert(var_name.into(), value.to_owned())
    }

    pub fn get_as_ref(&self, var_name: &str) -> Option<&Value> {
        self.0.get(var_name)
    }
}

fn eval_expr(expr: &Expr, vmap: &mut VarMap) -> Value {
    match expr {
        Expr::Aritmetic(e) => eval_arit_expr(e, vmap),
        Expr::Boolean(e) => eval_boolean_expr(e, vmap),
    }
}

fn eval_arit_expr(expr: &Expression, vmap: &mut VarMap) -> Value {
    let lhs: Value = match &expr.lhs {
        ExprTerm::Value(v) => v.clone(),
        ExprTerm::Node(n) => eval_arit_expr(&*n, vmap),
        ExprTerm::Variable(s) => vmap
            .get(s)
            .expect(format!("Undefined variable: {}", s).as_str()),
        ExprTerm::FnCall(func, args) => eval_fn(func, args, vmap).unwrap(),
    };
    let rhs: Value = match &expr.rhs {
        ExprTerm::Value(v) => v.clone(),
        ExprTerm::Node(n) => eval_arit_expr(&*n, vmap),
        ExprTerm::Variable(s) => vmap
            .get(s)
            .expect(format!("Undefined variable: {}", s).as_str()),
        ExprTerm::FnCall(func, args) => eval_fn(func, args, vmap).unwrap(),
    };

    expr.op.operate(&lhs, &rhs)
}

fn eval_boolean_expr(expr: &Expression, vmap: &mut VarMap) -> Value {
    let lhs: Value = match &expr.lhs {
        ExprTerm::Value(v) => v.clone(),
        ExprTerm::Node(n) => eval_boolean_expr(&*n, vmap),
        ExprTerm::Variable(s) => vmap
            .get(s)
            .expect(format!("Undefined variable: {}", s).as_str()),
        ExprTerm::FnCall(func, args) => eval_fn(func, args, vmap).unwrap(),
    };
    let rhs: Value = match &expr.rhs {
        ExprTerm::Value(v) => v.clone(),
        ExprTerm::Node(n) => eval_boolean_expr(&*n, vmap),
        ExprTerm::Variable(s) => vmap
            .get(s)
            .expect(format!("Undefined variable: {}", s).as_str()),
        ExprTerm::FnCall(func, args) => {
            eval_fn(func, args, vmap).unwrap() //TODO return result value
        }
    };

    expr.op.operate(&lhs, &rhs)
}

/// Helper for parsing Value from FnArgs
fn parse_fn_arg<'a>(
    args: &'a [FnArg],
    idx: usize,
    vmap: &'a VarMap,
) -> Result<Option<&'a Value>, RuntimeErr> {
    match &args.get(idx) {
        Some(FnArg::Value(s)) => Ok(Some(s)),
        Some(FnArg::Variable(s)) => {
            match &vmap.get_as_ref(s) {
                Some(v) => Ok(Some(*v)),
                None => Err(RuntimeErr::UndefinedVariable), // should be handled with parser
            }
        }
        _ => Ok(None),
    }
}

fn eval_fn(func: &Fn, args: &Vec<FnArg>, vmap: &VarMap) -> Result<Value, RuntimeErr> {
    match func.name {
        FnName::Concat => {
            let mut result = String::with_capacity(64);
            for i in 0..args.len() {
                // cannot be None coz we iterate by the array
                if let Value::String(s) = parse_fn_arg(&args, i, vmap)?.unwrap() {
                    result.push_str(&s)
                } else {
                    return Err(RuntimeErr::InvalidFunctionArgument);
                }
            }
            Ok(Value::String(result))
        }
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
        // TODO array len?
        _ => Err(RuntimeErr::InvalidFunctionArgument),
    }
}

#[cfg(test)]
mod test {

    use super::*;

    //TODO write test macros
    #[test]
    fn eval_fn_concat() {
        let args = vec![
            FnArg::Value(Value::String("this is".into())),
            FnArg::Value(Value::String(" my".into())),
            FnArg::Value(Value::String(" ".into())),
            FnArg::Value(Value::String("test".into())),
            FnArg::Value(Value::String(" fn".into())),
        ];

        let func = Fn {
            name: FnName::Concat,
            //args: vec![DataType::String],
            //variadic: true,
            ret_type: DataType::String,
        };

        let vmap = VarMap::new();
        let result = eval_fn(&func, &args, &vmap).unwrap();
        let expected = Value::String("this is my test fn".into());
        assert_eq!(result, expected);
    }

    #[test]
    fn eval_fn_in_str_true() {
        let args = vec![
            FnArg::Value(Value::String("future".into())),
            FnArg::Value(Value::String("future is near".into())),
        ];

        let func = Fn {
            name: FnName::InString,
            //args: vec![DataType::String],
            //variadic: true,
            ret_type: DataType::String,
        };

        let vmap = VarMap::new();
        let result = eval_fn(&func, &args, &vmap).unwrap();
        let expected = Value::Boolean(true);
        assert_eq!(result, expected);
    }

    #[test]
    fn eval_fn_variable_in_str_with_variable() {
        let args = vec![
            FnArg::Variable("some_var".into()),
            FnArg::Value(Value::String("future is near".into())),
        ];

        let func = Fn {
            name: FnName::InString,
            //args: vec![DataType::String],
            //variadic: true,
            ret_type: DataType::String,
        };

        let mut vmap = VarMap::new();
        vmap.set("some_var", &Value::String("future".into()));
        let result = eval_fn(&func, &args, &vmap).unwrap();
        let expected = Value::Boolean(true);
        assert_eq!(result, expected);
    }

    #[test]
    fn eval_fn_instr_false() {
        let args = vec![
            FnArg::Value(Value::String("futures".into())),
            FnArg::Value(Value::String("future is near".into())),
        ];

        let func = Fn {
            name: FnName::InString,
            //args: vec![DataType::String],
            //variadic: true,
            ret_type: DataType::String,
        };

        let vmap = VarMap::new();
        let result = eval_fn(&func, &args, &vmap).unwrap();
        let expected = Value::Boolean(false);
        assert_eq!(result, expected);
    }

    #[test]
    fn eval_fn_inarray_true() {
        let args = vec![
            FnArg::Value(Value::String("near".into())),
            FnArg::Value(Value::ArrString(vec![
                "test".into(),
                "near".into(),
                "function".into(),
            ])),
        ];

        let func = Fn {
            name: FnName::InArray,
            //args: vec![DataType::String],
            //variadic: true,
            ret_type: DataType::String,
        };

        let vmap = VarMap::new();
        let result = eval_fn(&func, &args, &vmap).unwrap();
        let expected = Value::Boolean(true);
        assert_eq!(result, expected);
    }

    #[test]
    fn eval_fn_inarray_false() {
        let args = vec![
            FnArg::Value(Value::String("nears".into())),
            FnArg::Value(Value::ArrString(vec![
                "test".into(),
                "near".into(),
                "function".into(),
            ])),
        ];

        let func = Fn {
            name: FnName::InArray,
            //args: vec![DataType::String],
            //variadic: true,
            ret_type: DataType::String,
        };

        let vmap = VarMap::new();
        let result = eval_fn(&func, &args, &vmap).unwrap();
        let expected = Value::Boolean(false);
        assert_eq!(result, expected);
    }

    #[test]
    fn eval_fn_arraypush_str() {
        let args = vec![
            FnArg::Value(Value::String("nears".into())),
            FnArg::Value(Value::ArrString(vec![
                "test".into(),
                "near".into(),
                "function".into(),
            ])),
        ];

        let func = Fn {
            name: FnName::ArrayPush,
            //args: vec![DataType::String],
            //variadic: true,
            ret_type: DataType::String,
        };

        let vmap = VarMap::new();
        let result = eval_fn(&func, &args, &vmap).unwrap();
        let expected = Value::ArrString(vec![
            "test".into(),
            "near".into(),
            "function".into(),
            "nears".into(),
        ]);
        assert_eq!(result, expected);
    }

    #[test]
    fn eval_fn_arraypush_int() {
        let args = vec![
            FnArg::Value(Value::Integer(3)),
            FnArg::Value(Value::ArrInteger(vec![3, 3, 3])),
        ];

        let func = Fn {
            name: FnName::ArrayPush,
            //args: vec![DataType::String],
            //variadic: true,
            ret_type: DataType::String,
        };
        let vmap = VarMap::new();
        let result = eval_fn(&func, &args, &vmap).unwrap();
        let expected = Value::ArrInteger(vec![3, 3, 3, 3]);
        assert_eq!(result, expected);
    }

    #[test]
    fn eval_fn_arraypop_str() {
        let args = vec![FnArg::Value(Value::ArrString(vec![
            "test".into(),
            "near".into(),
            "function".into(),
        ]))];

        let func = Fn {
            name: FnName::ArrayPop,
            //args: vec![DataType::String],
            //variadic: true,
            ret_type: DataType::String,
        };

        let vmap = VarMap::new();
        let result = eval_fn(&func, &args, &vmap).unwrap();
        let expected = Value::ArrString(vec!["test".into(), "near".into()]);
        assert_eq!(result, expected);
    }

    #[test]
    fn aritmetic_expr() {
        let mut vmap = VarMap::new();
        vmap.set("some_var", &Value::Integer(4));
        // 2 * (3 + a)
        let expr = Expr::Aritmetic(Expression {
            op: Operator::Aritmetic(AritmeticOperation::Multiply),
            lhs: ExprTerm::Value(Value::Integer(2)),
            rhs: ExprTerm::Node(Box::new(Expression {
                op: Operator::Aritmetic(AritmeticOperation::Add),
                lhs: ExprTerm::Value(Value::Integer(3)),
                rhs: ExprTerm::Variable("some_var".into()),
            })),
        });

        let result = eval_expr(&expr, &mut vmap);
        let expected = Value::Integer(14);
        assert_eq!(result, expected);
    }

    #[test]
    fn logic_expr_and() {
        let mut vmap = VarMap::new();
        vmap.set("a", &Value::Integer(5));
        vmap.set("b", &Value::Integer(1));
        // 2 * a == 10 && b < 2
        let expr = Expr::Boolean(Expression {
            op: Operator::Logic(LogicOperation::And),
            lhs: ExprTerm::Node(Box::new(Expression {
                op: Operator::Relational(RelationalOperation::Eqs),
                lhs: ExprTerm::Node(Box::new(Expression {
                    op: Operator::Aritmetic(AritmeticOperation::Multiply),
                    lhs: ExprTerm::Value(Value::Integer(2)),
                    rhs: ExprTerm::Variable("a".into()),
                })),
                rhs: ExprTerm::Value(Value::Integer(10)),
            })),
            rhs: ExprTerm::Node(Box::new(Expression {
                op: Operator::Relational(RelationalOperation::Lt),
                lhs: ExprTerm::Variable("b".into()),
                rhs: ExprTerm::Value(Value::Integer(2)),
            })),
        });

        let result = eval_expr(&expr, &mut vmap);
        let expected = Value::Boolean(true);
        assert_eq!(result, expected);
    }

    #[test]
    fn logic_expr_or_true() {
        // 2 * 5 == 11 || 1 < 2
        let mut vmap = VarMap::new();
        let expr = Expr::Boolean(Expression {
            op: Operator::Logic(LogicOperation::Or),
            lhs: ExprTerm::Node(Box::new(Expression {
                op: Operator::Relational(RelationalOperation::Eqs),
                lhs: ExprTerm::Node(Box::new(Expression {
                    op: Operator::Aritmetic(AritmeticOperation::Multiply),
                    lhs: ExprTerm::Value(Value::Integer(2)),
                    rhs: ExprTerm::Value(Value::Integer(5)),
                })),
                rhs: ExprTerm::Value(Value::Integer(11)),
            })),
            rhs: ExprTerm::Node(Box::new(Expression {
                op: Operator::Relational(RelationalOperation::Lt),
                lhs: ExprTerm::Value(Value::Integer(1)),
                rhs: ExprTerm::Value(Value::Integer(2)),
            })),
        });

        let result = eval_expr(&expr, &mut vmap);
        let expected = Value::Boolean(true);
        assert_eq!(result, expected);
    }

    #[test]
    fn logic_expr_or_false() {
        // 2 * 5 == 11 || 2 < 2
        let mut vmap = VarMap::new();
        let expr = Expr::Boolean(Expression {
            op: Operator::Logic(LogicOperation::Or),
            lhs: ExprTerm::Node(Box::new(Expression {
                op: Operator::Relational(RelationalOperation::Eqs),
                lhs: ExprTerm::Node(Box::new(Expression {
                    op: Operator::Aritmetic(AritmeticOperation::Multiply),
                    lhs: ExprTerm::Value(Value::Integer(2)),
                    rhs: ExprTerm::Value(Value::Integer(5)),
                })),
                rhs: ExprTerm::Value(Value::Integer(11)),
            })),
            rhs: ExprTerm::Node(Box::new(Expression {
                op: Operator::Relational(RelationalOperation::Lt),
                lhs: ExprTerm::Value(Value::Integer(2)),
                rhs: ExprTerm::Value(Value::Integer(2)),
            })),
        });

        let result = eval_expr(&expr, &mut vmap);
        let expected = Value::Boolean(false);
        assert_eq!(result, expected);
    }

    #[test]
    fn execute_program_simple_case() {
        /*
        Test case:
        let c = registr; //s0 (10)
        let b = input; //s1 (3)
        let a = 10; //s2
        if (a > input) { //s3 //c1
            c = b * 2 //s4 c1 = true
        } else {
            b = c * a //s5 c1 = false
        }
        d = a + b //s6
        output = d //s7
        */

        let steps = vec![
            Instruction::Statement(Statement {
                var_name: "c".into(),
                value: StatementVal::Register("rc".into()),
            }),
            Instruction::Statement(Statement {
                var_name: "b".into(),
                value: StatementVal::Input,
            }),
            Instruction::Statement(Statement {
                var_name: "a".into(),
                value: StatementVal::Const(Value::Integer(10)),
            }),
            Instruction::Condition(0),
            Instruction::Statement(Statement {
                var_name: "c".into(),
                value: StatementVal::Expr(Expr::Aritmetic(Expression {
                    op: Operator::Aritmetic(AritmeticOperation::Multiply),
                    lhs: ExprTerm::Variable("b".into()),
                    rhs: ExprTerm::Value(Value::Integer(2)),
                })),
            }),
            Instruction::Statement(Statement {
                var_name: "b".into(),
                value: StatementVal::Expr(Expr::Aritmetic(Expression {
                    op: Operator::Aritmetic(AritmeticOperation::Multiply),
                    lhs: ExprTerm::Variable("c".into()),
                    rhs: ExprTerm::Variable("a".into()),
                })),
            }),
            Instruction::Statement(Statement {
                var_name: "d".into(),
                value: StatementVal::Expr(Expr::Aritmetic(Expression {
                    op: Operator::Aritmetic(AritmeticOperation::Add),
                    lhs: ExprTerm::Variable("a".into()),
                    rhs: ExprTerm::Variable("b".into()),
                })),
            }),
            Instruction::Statement(Statement {
                var_name: "output".into(),
                value: StatementVal::Variable("d".into()),
            }),
        ];
        let conditions = vec![Condition {
            expr: ConditionExpr::Expr(Expr::Boolean(Expression {
                op: Operator::Relational(RelationalOperation::Gt),
                lhs: ExprTerm::Variable("a".into()),
                rhs: ExprTerm::Variable("input".into()),
            })),
            true_path: vec![4],  //instruction index
            false_path: vec![5], //instruction index
            ret: 6,              //instruction index
        }];

        let mut program = Program {
            instructions: steps,
            conditions: conditions,
            input: Value::Null,
            output: Value::Null,
        };

        let mut register_api = VarMap::new();
        register_api.set("rc", &Value::Integer(10));

        let mut interpreter = Interpreter::new();
        interpreter.vmap.set("input".into(), &Value::Integer(3));

        let result = interpreter.eval(&mut program, &mut register_api);
        let expected = Value::Integer(13);
        dbg!("RESULT STATE:");
        dbg!(interpreter.vmap);
        assert_eq!(result, expected);
    }

    #[test]
    fn execute_program_with_fn_in_array() {
        /*
        Test case:
        let y = registr // assume value (array['near', 'future', 'test', 'rust'])
        let x = input;  // input value 'test'
        if (in_array(x, y)) {
            output = true
            random_var = "just for testing"
        } else {
            output = false
        }
        */

        let steps = vec![
            Instruction::Statement(Statement {
                var_name: "y".into(),
                value: StatementVal::Register("register_with_array".into()),
            }),
            Instruction::Statement(Statement {
                var_name: "x".into(),
                value: StatementVal::Input,
            }),
            Instruction::Condition(0),
            Instruction::Statement(Statement {
                var_name: "output".into(),
                value: StatementVal::Const(Value::Boolean(true)),
            }),
            Instruction::Statement(Statement {
                var_name: "random_var".into(),
                value: StatementVal::Const(Value::String("just for testing".into())),
            }),
            Instruction::Statement(Statement {
                var_name: "output".into(),
                value: StatementVal::Const(Value::Boolean(false)),
            }),
        ];
        let conditions = vec![Condition {
            expr: ConditionExpr::FnCall(
                Fn {
                    name: FnName::InArray,
                    ret_type: DataType::Boolean,
                },
                vec![FnArg::Variable("x".into()), FnArg::Variable("y".into())],
            ),
            true_path: vec![3, 4], //instruction index
            false_path: vec![5],   //instruction index
            ret: 6,                //instruction index
        }];

        let mut program = Program {
            instructions: steps,
            conditions: conditions,
            input: Value::Null,
            output: Value::Null,
        };

        let mut register_api = VarMap::new();
        register_api.set(
            "register_with_array",
            &Value::ArrString(vec![
                "near".into(),
                "future".into(),
                "test".into(),
                "rust".into(),
            ]),
        );

        let mut interpreter = Interpreter::new();
        interpreter
            .vmap
            .set("input".into(), &Value::String("neco".into()));

        let result = interpreter.eval(&mut program, &mut register_api);
        let expected = Value::Boolean(false);
        dbg!("RESULT STATE:");
        dbg!(interpreter.vmap);
        assert_eq!(result, expected);
    }

    #[test]
    fn execute_program_with_fn_in_str() {
        /*
        Test case:
        let y = registr // assume value "future is near";
        let x = concat(input," ", "is"," ","near");  // input value: "future"
        if (x == y)) {
            output = true
        } else {
            output = false
        }
        */

        let steps = vec![
            Instruction::Statement(Statement {
                var_name: "y".into(),
                value: StatementVal::Register("some_register".into()),
            }),
            Instruction::Statement(Statement {
                var_name: "x".into(),
                value: StatementVal::FnCall(
                    Fn {
                        name: FnName::Concat,
                        ret_type: DataType::String,
                    },
                    vec![
                        FnArg::Variable("input".into()),
                        FnArg::Value(Value::String(" ".into())),
                        FnArg::Value(Value::String("is".into())),
                        FnArg::Value(Value::String(" ".into())),
                        FnArg::Value(Value::String("near".into())),
                    ],
                ),
            }),
            Instruction::Condition(0),
            Instruction::Statement(Statement {
                var_name: "output".into(),
                value: StatementVal::Const(Value::Boolean(true)),
            }),
            Instruction::Statement(Statement {
                var_name: "output".into(),
                value: StatementVal::Const(Value::Boolean(false)),
            }),
        ];
        let conditions = vec![Condition {
            expr: ConditionExpr::Expr(Expr::Boolean(Expression {
                op: Operator::Relational(RelationalOperation::Eqs),
                lhs: ExprTerm::Variable("x".into()),
                rhs: ExprTerm::Variable("y".into()),
            })),
            true_path: vec![3],  //instruction index
            false_path: vec![4], //instruction index
            ret: 5,              //instruction index
        }];

        let mut program = Program {
            instructions: steps,
            conditions: conditions,
            input: Value::Null,
            output: Value::Null,
        };

        let mut register_api = VarMap::new();
        register_api.set("some_register", &Value::String("future is near".into()));

        let mut interpreter = Interpreter::new();
        interpreter
            .vmap
            .set("input".into(), &Value::String("future".into()));

        let result = interpreter.eval(&mut program, &mut register_api);
        let expected = Value::Boolean(true);
        dbg!("RESULT STATE:");
        dbg!(interpreter.vmap);
        assert_eq!(result, expected);
    }
}

// DAO Register API - Prototype
trait RegisterStorage {
    fn fetch(&self, register: String) -> Value;
    fn store(&mut self, register: String, value: Value) -> Option<Value>;
    fn store_vec(&mut self, keys: &Vec<String>, values: &Vec<Value>);
}

/* struct RegisterProvider {

}

impl RegisterStorage for RegisterProvider {

} */

 */
