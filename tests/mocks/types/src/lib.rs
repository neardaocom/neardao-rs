#![allow(unused)]

use library::storage::StorageBucket;
use library::types::consts::Consts;
use library::types::datatype::Value;
use library::types::source::{Source, MutableSource, SourceDataVariant, SourceProvider};

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

    fn storage(&self, key: &str) -> Option<&Value> {
        todo!()
    }

    fn global_storage(&self, key: &str) -> Option<&Value> {
        todo!()
    }

    fn dao_const(&self, key: u8) -> Option<Value> {
        todo!()
    }

    fn props(&self, key: &str) -> Option<&Value> {
        todo!()
    }

    fn props_shared(&self, key: &str) -> Option<&Value> {
        todo!()
    }
}

impl MutableSource for SourceMock {
    fn replace_storage(&mut self, new: StorageBucket) -> Option<StorageBucket> {
        todo!()
    }

    fn set_prop(&mut self, new: SourceDataVariant) -> Option<SourceDataVariant> {
        todo!()
    }

    fn replace_settings(&mut self, new: SourceDataVariant) -> SourceDataVariant {
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

    fn unset_prop(&mut self) -> Option<SourceDataVariant> {
        todo!()
    }
}

impl Source for SourceMock {}
