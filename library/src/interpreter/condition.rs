use crate::types::datatype::Value;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
};

use super::{error::EvalError, expression::EExpr};

// TODO: Remove all Debug in production.

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Condition {
    pub expr: EExpr,
    pub true_path: u8,
    pub false_path: u8,
}

impl Condition {
    pub fn eval(&self, args: &[Value]) -> Result<u8, EvalError> {
        if let Value::Bool(v) = self.expr.eval(args)? {
            match v {
                true => Ok(self.true_path),
                false => Ok(self.false_path),
            }
        } else {
            Err(EvalError::InvalidDatatype)
        }
    }
}
