use crate::Value;

/// Trait for dynamically known values provider.
pub trait RuntimeConstantProvider {
    fn get(&self, key: u8) -> Option<Value>;
}
