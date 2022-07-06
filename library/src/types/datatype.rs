use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
};

use crate::ObjectId;

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
    OptionalObject(ObjectId),
    VecObject(ObjectId),
    Enum(Vec<ObjectId>),
    OptionalEnum(Vec<ObjectId>),
    VecEnum(Vec<ObjectId>),
    Tuple(ObjectId),
    OptionalTuple(ObjectId),
    VecTuple(ObjectId),
}

impl Datatype {
    pub fn is_optional(&self) -> bool {
        match self {
            Self::Bool(v) | Self::U64(v) | Self::U128(v) | Self::String(v) => *v,
            Self::OptionalObject(_) => true,
            _ => false,
        }
    }
}
