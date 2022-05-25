use near_sdk::IntoStorageKey;

use crate::{
    types::{error::SourceError, source::Source},
    workflow::types::Src,
};

pub(crate) fn object_key(prefix: &str, mid: &str, suffix: &str) -> String {
    let mut key = String::with_capacity(prefix.len() + mid.len() + suffix.len() + 2);
    key.push_str(prefix);
    key.push('.');
    key.push_str(mid);

    if !suffix.is_empty() {
        key.push('.');
        key.push_str(suffix);
    }
    key
}

pub fn append(key: &[u8], suffix: &[u8]) -> Vec<u8> {
    [key, suffix].concat()
}

pub fn into_storage_key_wrapper_u16(prefix: &[u8], id: u16) -> StorageKeyWrapper {
    append(prefix, &id.to_le_bytes()).into()
}

pub fn into_storage_key_wrapper_str(prefix: &[u8], id: &str) -> StorageKeyWrapper {
    append(prefix, &id.as_bytes()).into()
}

pub struct StorageKeyWrapper(pub Vec<u8>);

impl IntoStorageKey for StorageKeyWrapper {
    fn into_storage_key(self) -> Vec<u8> {
        self.0
    }
}

impl From<Vec<u8>> for StorageKeyWrapper {
    fn from(bytes: Vec<u8>) -> StorageKeyWrapper {
        StorageKeyWrapper(bytes)
    }
}

/// Helper function to fetch value reference from Source trait object.
pub fn get_value_from_source(
    sources: &dyn Source,
    src: &Src,
) -> Result<crate::types::datatype::Value, SourceError> {
    match src {
        Src::Tpl(key) => {
            let value = sources
                .tpl(key)
                .ok_or(SourceError::TplSettingValueMissing(key.into()))?
                .to_owned();
            Ok(value)
        }
        Src::TplSettings(key) => {
            let value = sources
                .tpl_settings(key)
                .ok_or(SourceError::TplSettingValueMissing(key.into()))?
                .to_owned();
            Ok(value)
        }
        Src::Action(key) => {
            let value = sources
                .props_action(key)
                .ok_or(SourceError::ActionValueMissing(key.into()))?
                .to_owned();
            Ok(value)
        }
        Src::Activity(key) => {
            let value = sources
                .props_shared(key)
                .ok_or(SourceError::ActivityValueMissing(key.into()))?
                .to_owned();
            Ok(value)
        }
        Src::Storage(key) => {
            let value = sources
                .storage(key)
                .ok_or(SourceError::StorageValueMissing(key.into()))?;
            Ok(value)
        }
        Src::GlobalStorage(key) => {
            let value = sources
                .global_storage(key)
                .ok_or(SourceError::GlobalStorageValueMissing(key.into()))?;
            Ok(value)
        }
        Src::Runtime(key) => {
            let value = sources
                .dao_const(*key)
                .ok_or(SourceError::RuntimeValueMissing(*key))?
                .to_owned();
            Ok(value)
        }
        Src::PropSettings(key) => {
            let value = sources
                .props_global(key)
                .ok_or(SourceError::ProposeValueMissing(key.into()))?
                .to_owned();
            Ok(value)
        }
        Src::User(_) => Err(SourceError::InvalidSourceVariant("user".into())),
    }
}

/// Calculates percents from given values.
/// Rounds > 0.5 up.
/// Invariants that must be uphold by caller:
/// - `total` >= value
/// - `total` <= 10^32
/// Panics or gives invalid results otherwise.
/// Tested in wasm32 environment via workspaces-rs in Sandbox and Testnet.
pub fn calculate_percent_u128(value: u128, total: u128) -> u8 {
    (((value * 10_000) / total) as f64 / 100.0).round() as u8
}
