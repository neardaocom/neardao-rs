use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    json_types::U128,
    serde::{Deserialize, Serialize},
};

use crate::ObjectId;

use super::error::TypeError;

//TODO remove debug in prod
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive())]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum Value {
    Bool(bool),
    U64(u64),
    #[serde(rename = "str_int")]
    U128(U128),
    Null,
    String(String),
    VecBool(Vec<bool>),
    VecU64(Vec<u64>),
    #[serde(rename = "vec_str_int")]
    VecU128(Vec<U128>),
    VecString(Vec<String>),
}

impl Default for Value {
    fn default() -> Self {
        Value::Null
    }
}

// TODO better error type
impl Value {
    pub fn try_into_bool(self) -> Result<bool, TypeError> {
        match self {
            Value::Bool(b) => Ok(b),
            _ => Err(TypeError::Conversion),
        }
    }

    pub fn try_into_string(self) -> Result<String, TypeError> {
        match self {
            Value::String(s) => Ok(s),
            _ => Err(TypeError::Conversion),
        }
    }

    pub fn try_into_u128(self) -> Result<u128, TypeError> {
        match self {
            Value::U64(n) => Ok(n as u128),
            Value::U128(n) => Ok(n.0),
            _ => Err(TypeError::Conversion),
        }
    }

    pub fn try_into_u64(self) -> Result<u64, TypeError> {
        match self {
            Value::U64(n) => Ok(n),
            Value::U128(n) => Ok(n.0 as u64),
            _ => Err(TypeError::Conversion),
        }
    }

    pub fn try_into_vec_string(self) -> Result<Vec<String>, TypeError> {
        match self {
            Value::VecString(v) => Ok(v),
            _ => Err(TypeError::Conversion),
        }
    }

    pub fn try_into_vec_u64(self) -> Result<Vec<u64>, TypeError> {
        match self {
            Value::VecU64(v) => Ok(v),
            _ => Err(TypeError::Conversion),
        }
    }

    pub fn try_into_str(&self) -> Result<&str, TypeError> {
        match self {
            Value::String(v) => Ok(v.as_str()),
            _ => Err(TypeError::Conversion),
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq))]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
/// Definition of expected datatype.
/// Bool in primitive datatypes denotes if its optional.
/// ObjectId references Metadata Object by pos.
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

#[allow(unused)]
#[cfg(test)]
mod test {
    use near_sdk::{json_types::U128, serde_json};

    use super::Value;
}
