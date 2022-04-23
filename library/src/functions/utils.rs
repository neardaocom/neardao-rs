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
