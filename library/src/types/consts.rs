use super::datatype::Value;

/// Trait for dynamically known values provider.
pub trait Consts {
    fn get(&self, key: u8) -> Option<Value>;
}
