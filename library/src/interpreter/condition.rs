use crate::types::DataType;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
};

use super::{error::EvalError, expression::EExpr};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Condition {
    pub expr: EExpr,
    pub true_path: u8,
    pub false_path: u8,
}

impl Condition {
    pub fn eval(&self, args: &[DataType]) -> Result<u8, EvalError> {
        if let DataType::Bool(v) = self.expr.eval(args)? {
            match v {
                true => Ok(self.true_path),
                false => Ok(self.false_path),
            }
        } else {
            Err(EvalError::InvalidType)
        }
    }
}
