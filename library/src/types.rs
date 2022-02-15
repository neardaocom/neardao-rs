use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    json_types::{U128, U64},
    serde::{Deserialize, Serialize},
};

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
    FnCallAdd,
    FnCallRemove,
    TagAdd,
    TagEdit,
    TagRemove,
    FtDistribute,
    TreasurySendFt,
    TreasurySendNft,
    TreasurySendNear,
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

//TODO remove debug in prod
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone,Debug)]
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
            DataType::U64(b) => Ok(b.0 as u128),
            DataType::U128(b) => Ok(b.0 as u128),
            _ => Err("DataType is not integer".into()),
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
/*  TODO remove
impl FnCallMetadata {

    /// Returns indexes of object with Vec objects
    pub fn vec_object_indexes(&self) -> Vec<u8> {
        self.arg_types
            .iter()
            .filter(|s| match s {
                DataTypeDef::VecObject(_) => true,
                _ => false,
            })
            .map(|s| match s {
                DataTypeDef::VecObject(e) => *e,
                _ => unreachable!(),
            })
            .collect()
    }
}
 */
