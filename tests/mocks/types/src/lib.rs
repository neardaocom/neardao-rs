#![allow(unused)]

use library::storage::StorageBucket;
use library::types::consts::Consts;
use library::types::datatype::Value;
use library::types::source::{MutableSource, Source, SourceDataVariant, SourceProvider};

pub struct SourceMock {
    pub tpls: Vec<(String, Value)>,
}

impl SourceProvider for SourceMock {
    fn tpl(&self, key: &str) -> Option<&Value> {
        self.tpls.iter().find(|el| el.0 == key).map(|el| &el.1)
    }

    fn tpl_settings(&self, key: &str) -> Option<&Value> {
        todo!()
    }

    fn storage(&self, key: &str) -> Option<Value> {
        todo!()
    }

    fn global_storage(&self, key: &str) -> Option<Value> {
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

    fn storage_mut(&mut self) -> Option<&mut StorageBucket> {
        todo!()
    }

    fn global_storage_mut(&mut self) -> Option<&mut StorageBucket> {
        todo!()
    }
}

impl MutableSource for SourceMock {
    fn replace_storage(&mut self, new: StorageBucket) -> Option<StorageBucket> {
        todo!()
    }

    fn set_prop_action(&mut self, new: SourceDataVariant) -> Option<SourceDataVariant> {
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

    fn set_prop_shared(&mut self, new: SourceDataVariant) -> Option<SourceDataVariant> {
        todo!()
    }

    fn unset_prop_action(&mut self) -> Option<SourceDataVariant> {
        todo!()
    }

    fn replace_global_storage(&mut self, new: StorageBucket) -> Option<StorageBucket> {
        todo!()
    }
}

impl Source for SourceMock {}
