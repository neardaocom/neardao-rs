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

use super::types::ArgSrc;
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
/// Expression wrapper for workflows
/// with defined sources for expression.
pub struct Expression {
    pub args: Vec<ArgSrc>,
    pub expr_id: u8,
}

impl Expression {
    pub fn bind_and_eval<S, A>(
        &self,
        sources: &S,
        input: &A,
        expressions: &[EExpr],
    ) -> Result<Value, ProcessingError>
    where
        S: Source + ?Sized,
        A: ActivityInput + ?Sized,
    {
        let expr = expressions
            .get(self.expr_id as usize)
            .ok_or(ProcessingError::MissingExpression)?;
        let mut binded_args: Vec<Value> = Vec::with_capacity(self.args.len());

        for arg_src in self.args.iter() {
            match arg_src {
                ArgSrc::User(key) => binded_args.push(
                    input
                        .get(key.as_str())
                        .expect("Failed to get value")
                        .clone(),
                ),
                _ => binded_args.push(get_value_from_source(sources, arg_src)?.to_owned()),
            }
        }
        Ok(expr.eval(binded_args.as_slice())?)
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum CondOrExpr {
    Cond(Condition),
    Expr(EExpr),
}
