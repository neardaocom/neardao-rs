use std::collections::HashMap;

use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
};

use crate::storage::StorageBucket;

use super::{consts::Consts, datatype::Value};

pub trait Source: SourceProvider + MutableSource {}
/// Trait representing possible `Value` sources for workflow.
pub trait SourceProvider {
    fn tpl(&self, key: &str) -> Option<&Value>;
    fn tpl_settings(&self, key: &str) -> Option<&Value>;
    fn props_global(&self, key: &str) -> Option<&Value>;
    fn props_action(&self, key: &str) -> Option<&Value>;
    fn props_shared(&self, key: &str) -> Option<&Value>;
    fn storage(&self, key: &str) -> Option<&Value>;
    fn global_storage(&self, key: &str) -> Option<&Value>;
    fn dao_const(&self, key: u8) -> Option<Value>;
}

pub trait MutableSource {
    fn replace_storage(&mut self, new: StorageBucket) -> Option<StorageBucket>;
    fn set_prop_shared(&mut self, new: SourceDataVariant) -> Option<SourceDataVariant>;
    fn unset_prop_action(&mut self) -> Option<SourceDataVariant>;
    fn set_prop_action(&mut self, new: SourceDataVariant) -> Option<SourceDataVariant>;
    fn replace_settings(&mut self, new: SourceDataVariant) -> Option<SourceDataVariant>;
    fn take_storage(&mut self) -> Option<StorageBucket>;
    fn take_global_storage(&mut self) -> Option<StorageBucket>;
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum SourceDataVariant {
    Map(HashMap<String, Value>),
}

impl SourceData for SourceDataVariant {
    fn get(&self, key: &str) -> Option<&Value> {
        match self {
            SourceDataVariant::Map(m) => m.get(key),
        }
    }

    fn set(&mut self, key: String, val: Value) -> Option<Value> {
        match self {
            SourceDataVariant::Map(m) => m.insert(key, val),
        }
    }

    fn take(&mut self, key: &str) -> Option<Value> {
        match self {
            SourceDataVariant::Map(m) => m.remove(key),
        }
    }
}

pub trait SourceData {
    fn get(&self, key: &str) -> Option<&Value>;
    fn set(&mut self, key: String, val: Value) -> Option<Value>;
    fn take(&mut self, key: &str) -> Option<Value>;
}

pub struct DefaultSource<T>
where
    T: Consts + Sized,
{
    tpls: SourceDataVariant,
    settings: Option<SourceDataVariant>,
    prop: Option<SourceDataVariant>,
    prop_action: Option<SourceDataVariant>,
    prop_shared: Option<SourceDataVariant>,
    dao_consts: T,
    storage: Option<StorageBucket>,
    global_storage: Option<StorageBucket>,
}

impl<T> SourceProvider for DefaultSource<T>
where
    T: Consts + Sized,
{
    fn tpl(&self, key: &str) -> Option<&Value> {
        self.tpls.get(key)
    }

    fn tpl_settings(&self, key: &str) -> Option<&Value> {
        if let Some(settings) = &self.settings {
            settings.get(key)
        } else {
            None
        }
    }

    fn storage(&self, key: &str) -> Option<&Value> {
        todo!()
    }

    fn global_storage(&self, key: &str) -> Option<&Value> {
        todo!()
    }

    fn dao_const(&self, key: u8) -> Option<Value> {
        self.dao_consts.get(key)
    }

    fn props_action(&self, key: &str) -> Option<&Value> {
        todo!()
    }

    fn props_shared(&self, key: &str) -> Option<&Value> {
        todo!()
    }
    fn props_global(&self, key: &str) -> Option<&Value> {
        if let Some(prop) = &self.prop {
            prop.get(key)
        } else {
            None
        }
    }
}

impl<T> MutableSource for DefaultSource<T>
where
    T: Consts + Sized,
{
    fn replace_storage(&mut self, new: StorageBucket) -> Option<StorageBucket> {
        if let Some(mut storage) = self.storage.as_mut() {
            Some(std::mem::replace(&mut storage, new))
        } else {
            self.storage = Some(new);
            None
        }
    }

    fn set_prop_action(&mut self, new: SourceDataVariant) -> Option<SourceDataVariant> {
        if let Some(mut prop) = self.prop.as_mut() {
            Some(std::mem::replace(&mut prop, new))
        } else {
            self.prop = Some(new);
            None
        }
    }

    fn set_prop_shared(&mut self, new: SourceDataVariant) -> Option<SourceDataVariant> {
        todo!()
    }

    fn replace_settings(&mut self, new: SourceDataVariant) -> Option<SourceDataVariant> {
        if let Some(mut settings) = self.settings.as_mut() {
            Some(std::mem::replace(&mut settings, new))
        } else {
            self.settings = Some(new);
            None
        }
    }

    fn take_storage(&mut self) -> Option<StorageBucket> {
        self.storage.take()
    }

    fn take_global_storage(&mut self) -> Option<StorageBucket> {
        self.global_storage.take()
    }

    fn unset_prop_action(&mut self) -> Option<SourceDataVariant> {
        std::mem::take(&mut self.prop_action)
    }
}

impl<T> Source for DefaultSource<T> where T: Consts {}

impl<T> DefaultSource<T>
where
    T: Consts,
{
    pub fn from(
        tpls: SourceDataVariant,
        settings: Option<SourceDataVariant>,
        prop: Option<SourceDataVariant>,
        dao_consts: T,
        storage: Option<StorageBucket>,
        global_storage: Option<StorageBucket>,
    ) -> Self {
        Self {
            tpls,
            settings,
            prop,
            prop_action: None,
            prop_shared: None,
            dao_consts,
            storage,
            global_storage,
        }
    }
}

#[cfg(test)]
pub struct SourceMock {
    pub tpls: Vec<(String, Value)>,
}

#[allow(unused)]
#[cfg(test)]
impl SourceProvider for SourceMock {
    fn tpl(&self, key: &str) -> Option<&Value> {
        self.tpls.iter().find(|el| el.0 == key).map(|el| &el.1)
    }
    fn tpl_settings(&self, key: &str) -> Option<&Value> {
        todo!()
    }
    fn storage(&self, key: &str) -> Option<&Value> {
        todo!()
    }
    fn global_storage(&self, key: &str) -> Option<&Value> {
        todo!()
    }
    fn dao_const(&self, key: u8) -> Option<Value> {
        todo!()
    }
    fn props_action(&self, key: &str) -> Option<&Value> {
        todo!()
    }
    fn props_shared(&self, key: &str) -> Option<&Value> {
        todo!()
    }

    fn props_global(&self, key: &str) -> Option<&Value> {
        todo!()
    }
}

#[allow(unused)]
#[cfg(test)]
impl MutableSource for SourceMock {
    fn replace_storage(&mut self, new: StorageBucket) -> Option<StorageBucket> {
        todo!()
    }
    fn set_prop_action(&mut self, new: SourceDataVariant) -> Option<SourceDataVariant> {
        todo!()
    }
    fn set_prop_shared(&mut self, new: SourceDataVariant) -> Option<SourceDataVariant> {
        todo!()
    }
    fn replace_settings(&mut self, new: SourceDataVariant) -> Option<SourceDataVariant> {
        todo!()
    }
    fn take_storage(&mut self) -> Option<StorageBucket> {
        todo!()
    }
    fn take_global_storage(&mut self) -> Option<StorageBucket> {
        todo!()
    }

    fn unset_prop_action(&mut self) -> Option<SourceDataVariant> {
        todo!()
    }
}

#[allow(unused)]
#[cfg(test)]
impl Source for SourceMock {}
