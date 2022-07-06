use std::collections::HashMap;

use library::{
    types::Value,
    workflow::{
        action::{ActionInput, ActionInputType},
        runtime::activity_input::UserInput,
        types::DaoActionIdent,
    },
};

/// Activity inputs for `Media1`.
pub struct ActivityInputMedia1;
impl ActivityInputMedia1 {
    pub fn activity_1() -> Vec<Option<ActionInput>> {
        let map = HashMap::new();
        vec![Some(ActionInput {
            action: ActionInputType::DaoAction(DaoActionIdent::MediaAdd),
            values: UserInput::Map(map),
        })]
    }
    pub fn activity_2() -> Vec<Option<ActionInput>> {
        let map = HashMap::new();
        vec![Some(ActionInput {
            action: ActionInputType::DaoAction(DaoActionIdent::MediaUpdate),
            values: UserInput::Map(map),
        })]
    }
    pub fn propose_settings_activity_1_cid() -> HashMap<String, Value> {
        let mut map = HashMap::new();
        map.insert("name".into(), Value::String("cid media name".into()));
        map.insert("category".into(), Value::String("test doc".into()));
        map.insert("type.cid.ipfs".into(), Value::String("web3.storage".into()));
        map.insert(
            "type.cid.cid".into(),
            Value::String("1234567890asdfghjkl".into()),
        );
        map.insert(
            "type.cid.mimetype".into(),
            Value::String("application/pdf".into()),
        );
        map.insert("tags".into(), Value::VecU64(vec![0, 1, 2]));
        map.insert("version".into(), Value::String("1.0".into()));
        map.insert("valid".into(), Value::Bool(true));
        map
    }
    pub fn propose_settings_activity_1_text() -> HashMap<String, Value> {
        let mut map = HashMap::new();
        map.insert("name".into(), Value::String("text media name".into()));
        map.insert("category".into(), Value::String("test doc".into()));
        map.insert(
            "type.text.0".into(),
            Value::String("blablabla very important text blablabla".into()),
        );
        map.insert("tags".into(), Value::VecU64(vec![0, 1, 2]));
        map.insert("version".into(), Value::String("1.0".into()));
        map.insert("valid".into(), Value::Bool(true));
        map
    }
    pub fn propose_settings_activity_1_link() -> HashMap<String, Value> {
        let mut map = HashMap::new();
        map.insert("name".into(), Value::String("link media name".into()));
        map.insert("category".into(), Value::String("test doc".into()));
        map.insert(
            "type.link.0".into(),
            Value::String("blablabla very important link blablabla".into()),
        );
        map.insert("tags".into(), Value::VecU64(vec![0, 1, 2]));
        map.insert("version".into(), Value::String("1.0".into()));
        map.insert("valid".into(), Value::Bool(true));
        map
    }
    pub fn propose_settings_activity_2_cid() -> HashMap<String, Value> {
        let mut map = HashMap::new();
        map.insert("id".into(), Value::U64(1));
        map.insert(
            "media.name".into(),
            Value::String("cid media name UPDATED".into()),
        );
        map.insert("media.category".into(), Value::String("test doc".into()));
        map.insert(
            "media.type.cid.ipfs".into(),
            Value::String("web3.storage".into()),
        );
        map.insert(
            "media.type.cid.cid".into(),
            Value::String("1234567890asdfghjkl".into()),
        );
        map.insert(
            "media.type.cid.mimetype".into(),
            Value::String("application/pdf".into()),
        );
        map.insert("media.tags".into(), Value::VecU64(vec![0, 1, 2]));
        map.insert("media.version".into(), Value::String("1.0".into()));
        map.insert("media.valid".into(), Value::Bool(true));
        map
    }
}
