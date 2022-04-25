use near_sdk::IntoStorageKey;

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
