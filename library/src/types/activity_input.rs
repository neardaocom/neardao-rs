use std::collections::HashMap;

use near_sdk::serde::Deserialize;

#[cfg(not(target_arch = "wasm32"))]
use near_sdk::serde::Serialize;

use super::datatype::Value;
/// Trait representing user input values for an activity.
pub trait ActivityInput {
    fn has_key(&self, key: &str) -> bool;
    fn get(&self, key: &str) -> Option<&Value>;
    fn set(&mut self, key: &str, val: Value);
    fn take(&mut self, key: &str) -> Option<Value>;
    fn remove(&mut self, key: &str) -> Option<Value>;
    fn to_vec(&self) -> Vec<(String, Value)>;
}

#[derive(Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, Serialize))]
#[serde(crate = "near_sdk::serde")]
pub enum UserInput {
    Map(InputHashMap),
}

impl UserInput {
    pub fn into_activity_input(self) -> Box<dyn ActivityInput> {
        match self {
            Self::Map(map) => Box::new(map),
        }
    }
}

#[derive(Deserialize, Default)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, Serialize))]
#[serde(crate = "near_sdk::serde")]
pub struct InputHashMap(pub HashMap<String, Value>);
impl InputHashMap {
    pub fn new() -> Self {
        InputHashMap(HashMap::new())
    }
}

impl ActivityInput for InputHashMap {
    fn get(&self, key: &str) -> Option<&Value> {
        self.0.get(key)
    }

    fn set(&mut self, key: &str, val: Value) {
        self.0.insert(key.to_string(), val);
    }

    fn take(&mut self, key: &str) -> Option<Value> {
        self.0.insert(key.to_owned(), Value::default())
    }

    fn remove(&mut self, key: &str) -> Option<Value> {
        self.0.remove(key)
    }

    fn has_key(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }

    fn to_vec(&self) -> Vec<(String, Value)> {
        self.0.clone().into_iter().collect()
    }
}
