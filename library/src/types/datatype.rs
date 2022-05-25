use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    json_types::U128,
    serde::{Deserialize, Serialize},
};

use crate::ObjectId;

use super::error::CastError;

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
            _ => Err(CastError::new(self.datatype(), "u128")),
        }
    }

    pub fn try_into_u64(&self) -> Result<u64, CastError> {
        match self {
            Value::U64(n) => Ok(*n),
            Value::U128(n) => Ok(n.0 as u64),
            _ => Err(CastError::new(self.datatype(), "u64")),
        }
    }

    pub fn try_into_vec_string(self) -> Result<Vec<String>, CastError> {
        match self {
            Value::VecString(v) => Ok(v),
            _ => Err(CastError::new(self.datatype(), "vec_string")),
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

// TODO: Maybe define new Datatype::Enum ??
/// Definition of expected datatype.
/// Bool in primitive datatypes denotes if its optional.
/// ObjectId references Metadata Object by pos.
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq))]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum Datatype {
    Bool(bool),
    U64(bool),
    U128(bool),
    String(bool),
    VecU64,
    VecU128,
    VecString,
    Object(ObjectId),
    NullableObject(ObjectId),
    VecObject(ObjectId),
}

impl Datatype {
    pub fn is_optional(&self) -> bool {
        match self {
            Self::Bool(v) | Self::U64(v) | Self::U128(v) | Self::String(v) => *v,
            Self::NullableObject(_) => true,
            _ => false,
        }
    }
}
