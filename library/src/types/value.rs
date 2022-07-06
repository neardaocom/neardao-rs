use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    json_types::U128,
    serde::{Deserialize, Serialize},
};

use crate::error::CastError;

// TODO: Remove debug in production.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive())]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum Value {
    Bool(bool),
    U64(u64),
    U128(U128),
    Null,
    String(String),
    VecBool(Vec<bool>),
    VecU64(Vec<u64>),
    VecU128(Vec<U128>),
    VecString(Vec<String>),
}

impl Default for Value {
    fn default() -> Self {
        Value::Null
    }
}

impl Value {
    pub fn datatype(&self) -> &str {
        match self {
            Value::Bool(_) => "bool",
            Value::U64(_) => "u64",
            Value::U128(_) => "U128",
            Value::String(_) => "string",
            Value::VecBool(_) => "vec_bool",
            Value::VecU64(_) => "vec_u64",
            Value::VecU128(_) => "vec_u128",
            Value::VecString(_) => "vec_string",
            Value::Null => "null",
        }
    }
}

impl Value {
    pub fn try_into_bool(&self) -> Result<bool, CastError> {
        match self {
            Value::Bool(b) => Ok(*b),
            _ => Err(CastError::new(self.datatype(), "bool")),
        }
    }

    pub fn try_into_string(self) -> Result<String, CastError> {
        match self {
            Value::String(s) => Ok(s),
            _ => Err(CastError::new(self.datatype(), "string")),
        }
    }

    pub fn try_into_u128(&self) -> Result<u128, CastError> {
        match self {
            Value::U64(n) => Ok(*n as u128),
            Value::U128(n) => Ok(n.0),
            Value::String(v) => Ok(v
                .parse::<u128>()
                .map_err(|_| CastError::new("string", "u128"))?),
            _ => Err(CastError::new(self.datatype(), "u128")),
        }
    }

    pub fn try_into_u64(&self) -> Result<u64, CastError> {
        match self {
            Value::U64(n) => Ok(*n),
            Value::U128(n) => Ok(n.0 as u64),
            Value::String(v) => Ok(v
                .parse::<u64>()
                .map_err(|_| CastError::new("string", "u64"))?),
            _ => Err(CastError::new(self.datatype(), "u64")),
        }
    }

    pub fn try_into_vec_string(self) -> Result<Vec<String>, CastError> {
        match self {
            Value::VecString(v) => Ok(v),
            _ => Err(CastError::new(self.datatype(), "vec_string")),
        }
    }
    pub fn try_into_vec_u128(self) -> Result<Vec<U128>, CastError> {
        match self {
            Value::VecU128(v) => Ok(v),
            _ => Err(CastError::new(self.datatype(), "vec_u128")),
        }
    }

    pub fn try_into_vec_u64(self) -> Result<Vec<u64>, CastError> {
        match self {
            Value::VecU64(v) => Ok(v),
            _ => Err(CastError::new(self.datatype(), "vec_u64")),
        }
    }

    pub fn try_into_str(&self) -> Result<&str, CastError> {
        match self {
            Value::String(v) => Ok(v.as_str()),
            _ => Err(CastError::new(self.datatype(), "&str")),
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }
}
