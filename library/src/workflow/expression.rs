use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
};

use crate::{
    functions::utils::get_value_from_source,
    interpreter::{condition::Condition, expression::EExpr},
    types::Value,
};

use super::{
    error::ProcessingError, runtime::activity_input::ActivityInput, runtime::source::Source,
    types::Src,
};

// TODO: Remove Debug in production.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq))]
#[serde(crate = "near_sdk::serde")]
/// Expression wrapper for workflows
/// with defined sources for expression.
pub struct Expression {
    pub args: Vec<Src>,
    pub expr_id: u8,
}

impl Expression {
    pub fn bind_and_eval(
        &self,
        sources: &dyn Source,
        input: Option<&dyn ActivityInput>,
        expressions: &[EExpr],
    ) -> Result<Value, ProcessingError>
where {
        let expr = expressions
            .get(self.expr_id as usize)
            .ok_or(ProcessingError::MissingExpression(self.expr_id))?;
        let mut binded_args: Vec<Value> = Vec::with_capacity(self.args.len());

        for src in self.args.iter() {
            let value = match src {
                Src::Input(key) => {
                    if let Some(user_input) = input {
                        user_input
                            .get(key.as_str())
                            .ok_or(ProcessingError::MissingInputKey(key.into()))?
                            .clone()
                    } else {
                        return Err(ProcessingError::InputNotProvided);
                    }
                }
                _ => get_value_from_source(sources, src)?,
            };
            binded_args.push(value);
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
