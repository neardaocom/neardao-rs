use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
};

use crate::{
    functions::get_value_from_source,
    interpreter::{condition::Condition, expression::EExpr},
    types::{error::ProcessingError, DataType},
};

use super::types::{ArgSrc, ValueContainer};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
/// Interpreter wrapper for workflows.
pub struct Expression {
    pub args: Vec<ArgSrc>,
    pub expr: EExpr,
}

impl Expression {
    pub fn bind_and_eval<T: std::convert::AsRef<[DataType]>>(
        &self,
        sources: &ValueContainer<T>,
        args: &[DataType],
    ) -> Result<DataType, ProcessingError> {
        let mut binded_args: Vec<DataType> = Vec::with_capacity(args.len());

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
        Ok(self.expr.eval(&mut binded_args)?)
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum CondOrExpr {
    Cond(Condition),
    Expr(EExpr),
}
