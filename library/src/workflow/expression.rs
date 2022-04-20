use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
};

use crate::{
    functions::binding::get_value_from_source,
    interpreter::{condition::Condition, expression::EExpr},
    types::{
        activity_input::ActivityInput, datatype::Value, error::ProcessingError, source::Source,
    },
};

use super::types::{ArgSrc, ArgSrcNew, ValueContainer};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
/// Interpreter wrapper for workflows.
pub struct Expression {
    pub args: Vec<ArgSrc>,
    pub expr: EExpr,
}

impl Expression {
    pub fn bind_and_eval<T: std::convert::AsRef<[Value]>>(
        &self,
        sources: &ValueContainer<T>,
        args: &[Value],
    ) -> Result<Value, ProcessingError> {
        let mut binded_args: Vec<Value> = Vec::with_capacity(args.len());

        for arg_src in self.args.iter() {
            match arg_src {
                ArgSrc::User(id) => binded_args.push(
                    args.get(*id as usize)
                        .ok_or(ProcessingError::UserInput(*id))?
                        .clone(),
                ),
                _ => binded_args.push(get_value_from_source(arg_src, sources)?),
            }
        }
        Ok(self.expr.eval(binded_args.as_slice())?)
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
/// Interpreter wrapper for workflows.
pub struct ExpressionNew {
    pub args: Vec<ArgSrcNew>,
    pub expr: EExpr,
}

impl ExpressionNew {
    pub fn bind_and_eval<S, A>(&self, sources: &S, args: &A) -> Result<Value, ProcessingError>
    where
        S: Source + ?Sized,
        A: ActivityInput + ?Sized,
    {
        let mut binded_args: Vec<Value> = Vec::with_capacity(self.args.len());

        for arg_src in self.args.iter() {
            match arg_src {
                ArgSrcNew::User(key) => {
                    binded_args.push(args.get(key.as_str()).expect("Failed to get value").clone())
                }
                ArgSrcNew::ConstsTpl(key) => binded_args.push(
                    sources
                        .get_tpl_const(key.as_str())
                        .expect("Failed to get value")
                        .clone(),
                ),
                _ => {
                    unimplemented!("Bind and eval is implemented only for User and ConstTpl inputs")
                }
            }
        }
        Ok(self.expr.eval(binded_args.as_slice())?)
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum CondOrExpr {
    Cond(Condition),
    Expr(EExpr),
}
