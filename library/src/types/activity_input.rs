use std::collections::HashMap;

use near_sdk::serde::{Deserialize, Serialize};

use super::datatype::Value;

pub trait ActivityInput {
    fn get(&self, key: &str) -> Option<&Value>;
    fn set(&mut self, key: &str, val: Value);
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub enum ValueCollection {
    Map(InputHashMap),
}

impl ValueCollection {
    pub fn into_activity_input(self) -> Box<dyn ActivityInput> {
        match self {
            Self::Map(map) => Box::new(map),
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct InputHashMap(pub HashMap<String, Value>);

impl ActivityInput for InputHashMap {
    fn get(&self, key: &str) -> Option<&Value> {
        self.0.get(key)
    }

    fn set(&mut self, key: &str, val: Value) {
        self.0.insert(key.to_string(), val);
    }
}
