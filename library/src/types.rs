use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
};

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum ActionIdent {
    GroupAdd,
    GroupRemove,
    GroupUpdate,
    GroupMemberAdd,
    GroupMemberRemove,
    SettingsUpdate,
    MediaAdd,
    MediaInvalidate,
    FnCall,
    FnCallAdd,
    FnCallRemove,
    TagAdd,
    TagEdit,
    TagRemove,
    FtUnlock,
    FtDistribute,
    FtSend,
    NftSend,
    NearSend,
    WorkflowAdd,
    WorkflowChange,
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

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum DataType {
    Null,
    String(String),
    Bool(bool),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    VecString(Vec<String>),
    VecBool(Vec<bool>),
    VecU8(Vec<u8>),
    VecU16(Vec<u16>),
    VecU32(Vec<u32>),
    VecU64(Vec<u64>),
    VecU128(Vec<u128>),
}

impl DataType {
    pub fn try_into_bool(self) -> Result<bool, String> {
        match self {
            DataType::Bool(b) => Ok(b),
            _ => Err("DataType is not bool".into()),
        }
    }

    pub fn try_into_string(self) -> Result<String, String> {
        match self {
            DataType::String(b) => Ok(b),
            _ => Err("DataType is not string".into()),
        }
    }

    pub fn try_into_u128(self) -> Result<u128, String> {
        match self {
            DataType::U8(b) => Ok(b as u128),
            DataType::U16(b) => Ok(b as u128),
            DataType::U32(b) => Ok(b as u128),
            DataType::U64(b) => Ok(b as u128),
            DataType::U128(b) => Ok(b as u128),
            _ => Err("DataType is not string".into()),
        }
    }
}
