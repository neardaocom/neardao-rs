use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    json_types::{U128, U64},
    serde::{Deserialize, Serialize},
};

use crate::ObjectId;

use self::error::TypeError;

pub mod error;

//TODO remove debug in prod
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive())]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum DataType {
    Bool(bool),
    #[serde(rename = "int")]
    U64(u64),
    #[serde(rename = "str_int")]
    U128(U128),
    Null,
    String(String),
    VecBool(Vec<bool>),
    #[serde(rename = "vec_int")]
    VecU64(Vec<u64>),
    #[serde(rename = "vec_str_int")]
    VecU128(Vec<U128>),
    VecString(Vec<String>),
}

// TODO better error type
impl DataType {
    pub fn try_into_bool(self) -> Result<bool, TypeError> {
        match self {
            DataType::Bool(b) => Ok(b),
            _ => Err(TypeError::Conversion),
        }
    }

    pub fn try_into_string(self) -> Result<String, TypeError> {
        match self {
            DataType::String(s) => Ok(s),
            _ => Err(TypeError::Conversion),
        }
    }

    pub fn try_into_u128(self) -> Result<u128, TypeError> {
        match self {
            DataType::U64(n) => Ok(n as u128),
            DataType::U128(n) => Ok(n.0),
            _ => Err(TypeError::Conversion),
        }
    }

    pub fn try_into_u64(self) -> Result<u64, TypeError> {
        match self {
            DataType::U64(n) => Ok(n),
            DataType::U128(n) => Ok(n.0 as u64),
            _ => Err(TypeError::Conversion),
        }
    }

    pub fn try_into_vec_str(self) -> Result<Vec<String>, TypeError> {
        match self {
            DataType::VecString(v) => Ok(v),
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
pub enum DataTypeDef {
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

    use super::DataType;
}
