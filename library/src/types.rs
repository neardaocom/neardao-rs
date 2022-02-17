use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    json_types::{U128, U64},
    serde::{Deserialize, Serialize},
};

use crate::{EventCode, FnCallId};

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum ValidatorType {
    Primitive(u8),
    Collection(u8),
}

impl ValidatorType {
    pub fn get_id(&self) -> u8 {
        match self {
            ValidatorType::Primitive(id) => *id,
            ValidatorType::Collection(id) => *id,
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum ActionIdent {
    GroupAdd,
    GroupRemove,
    GroupUpdate,
    GroupAddMembers,
    GroupRemoveMember,
    SettingsUpdate,
    MediaAdd,
    MediaInvalidate,
    MediaRemove,
    FnCall,
    TagAdd,
    TagEdit,
    TagRemove,
    FtDistribute,
    TreasurySendFt,
    TreasurySendFtContract,
    TreasurySendNft,
    TreasurySendNFtContract,
    TreasurySendNear,
    WorkflowAdd,
    Event,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq, Clone))]
#[serde(crate = "near_sdk::serde")]
pub enum ActionData {
    FnCall(FnCallData),
    Event(EventData),
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq, Clone))]
#[serde(crate = "near_sdk::serde")]
pub struct FnCallData {
    pub id: FnCallId,
    pub tgas: u16,
    pub deposit: U128,
}
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq, Clone))]
#[serde(crate = "near_sdk::serde")]
pub struct EventData {
    pub code: EventCode,
    pub values: Vec<DataTypeDef>,
    pub deposit_from_bind: Option<u8>,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum DataTypeDef {
    String(bool),
    Bool(bool),
    U8(bool),
    U16(bool),
    U32(bool),
    U64(bool),
    U128(bool),
    VecString,
    VecU8,
    VecU16,
    VecU32,
    VecU64,
    VecU128,
    Object(u8), // 0 value in object means optional object
    NullableObject(u8),
    VecObject(u8),
}

//TODO remove debug in prod
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum DataType {
    Null,
    String(String),
    Bool(bool),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(U64),
    U128(U128),
    VecString(Vec<String>),
    VecBool(Vec<bool>),
    VecU8(Vec<u8>),
    VecU16(Vec<u16>),
    VecU32(Vec<u32>),
    VecU64(Vec<U64>),
    VecU128(Vec<U128>),
}

// TODO better error type
impl DataType {
    pub fn try_into_bool(self) -> Result<bool, String> {
        match self {
            DataType::Bool(b) => Ok(b),
            _ => Err("DataType is not bool".into()),
        }
    }

    pub fn try_into_string(self) -> Result<String, String> {
        match self {
            DataType::String(s) => Ok(s),
            _ => Err("DataType is not string".into()),
        }
    }

    pub fn try_into_u128(self) -> Result<u128, String> {
        match self {
            DataType::U8(n) => Ok(n as u128),
            DataType::U16(n) => Ok(n as u128),
            DataType::U32(n) => Ok(n as u128),
            DataType::U64(n) => Ok(n.0 as u128),
            DataType::U128(n) => Ok(n.0 as u128),
            _ => Err("DataType is not integer".into()),
        }
    }

    pub fn try_into_vec_str(self) -> Result<Vec<String>, String> {
        match self {
            DataType::VecString(v) => Ok(v),
            _ => Err("DataType is not VecString".into()),
        }
    }
}

// Represents object schema
// Coz compiler yelling at me: "error[E0275]: overflow evaluating the requirement" on Borsh we do it this way
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq, Clone))]
#[serde(crate = "near_sdk::serde")]
pub struct FnCallMetadata {
    pub arg_names: Vec<String>,
    pub arg_types: Vec<DataTypeDef>,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum VoteScenario {
    Democratic,
    TokenWeighted,
}
