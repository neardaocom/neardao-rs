use near_sdk::IntoStorageKey;

use crate::{
    types::{error::SourceError, source::Source},
    workflow::types::ArgSrc,
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

// TODO: Fix error messages.
/// Helper function to fetch value ref from Source.
pub fn get_value_from_source(
    sources: &dyn Source,
    src: &ArgSrc,
) -> Result<crate::types::datatype::Value, SourceError> {
    match src {
        ArgSrc::ConstsTpl(key) => {
            let value = sources
                .tpl(key)
                .ok_or(SourceError::SourceMissing("const tpl".into()))?
                .to_owned();
            Ok(value)
        }
        ArgSrc::ConstsSettings(key) => {
            let value = sources
                .tpl_settings(key)
                .ok_or(SourceError::SourceMissing("const tpl settings".into()))?
                .to_owned();
            Ok(value)
        }
        ArgSrc::ConstAction(key) => {
            let value = sources
                .props_action(key)
                .ok_or(SourceError::SourceMissing("const action".into()))?
                .to_owned();
            Ok(value)
        }
        ArgSrc::ConstActivityShared(key) => {
            let value = sources
                .props_shared(key)
                .ok_or(SourceError::SourceMissing("const activity shared".into()))?
                .to_owned();
            Ok(value)
        }
        ArgSrc::Storage(key) => {
            let value = sources
                .storage(key)
                .ok_or(SourceError::SourceMissing(format!(
                    "storage - key: {}",
                    key
                )))?;
            Ok(value)
        }
        ArgSrc::GlobalStorage(key) => {
            let value = sources
                .global_storage(key)
                .ok_or(SourceError::SourceMissing("global storage".into()))?;
            Ok(value)
        }
        ArgSrc::Const(key) => {
            let value = sources
                .dao_const(*key)
                .ok_or(SourceError::SourceMissing("dao const".into()))?
                .to_owned();
            Ok(value)
        }
        ArgSrc::ConstPropSettings(key) => {
            let value = sources
                .props_global(key)
                .ok_or(SourceError::SourceMissing(format!(
                    "const global prop - key: {}",
                    key
                )))?
                .to_owned();
            Ok(value)
        }
        _ => Err(SourceError::InvalidSourceVariant),
    }
}
