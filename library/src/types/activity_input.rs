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

// TODO: Remove Debug and Clone in production.
#[derive(Deserialize, Debug, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize))]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum UserInput {
    Map(HashMap<String, Value>),
}

impl UserInput {
    pub fn into_activity_input(self) -> Box<dyn ActivityInput> {
        match self {
            Self::Map(map) => Box::new(map),
        }
    }
}

impl ActivityInput for HashMap<String, Value> {
    fn get(&self, key: &str) -> Option<&Value> {
        self.get(key)
    }

    fn set(&mut self, key: &str, val: Value) {
        self.insert(key.to_string(), val);
    }

    fn take(&mut self, key: &str) -> Option<Value> {
        self.insert(key.to_owned(), Value::default())
    }

    fn remove(&mut self, key: &str) -> Option<Value> {
        self.remove(key)
    }

    fn has_key(&self, key: &str) -> bool {
        self.contains_key(key)
    }

    fn to_vec(&self) -> Vec<(String, Value)> {
        self.clone().into_iter().collect()
    }
}
