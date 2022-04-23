use super::datatype::Value;
/// Trait representing possible `Value` sources for workflow.
pub trait Source {
    fn tpl(&self, key: &str) -> Option<&Value>;
    fn tpl_settings(&self, key: &str) -> Option<&Value>;
    fn prop_settings(&self, key: &str) -> Option<&Value>;
    fn storage(&self, key: &str) -> Option<&Value>;
    fn global_storage(&self, key: &str) -> Option<&Value>;
    fn dao_const(&self, key: u8) -> Option<&Value>;
}

#[cfg(test)]
pub struct SourceMock {
    pub tpls: Vec<(String, Value)>,
}

#[allow(unused)]
#[cfg(test)]
impl Source for SourceMock {
    fn tpl(&self, key: &str) -> Option<&Value> {
        self.tpls.iter().find(|el| el.0 == key).map(|el| &el.1)
    }

    fn tpl_settings(&self, key: &str) -> Option<&Value> {
        todo!()
    }

    fn prop_settings(&self, key: &str) -> Option<&Value> {
        todo!()
    }

    fn storage(&self, key: &str) -> Option<&Value> {
        todo!()
    }

    fn global_storage(&self, key: &str) -> Option<&Value> {
        todo!()
    }

    fn dao_const(&self, key: u8) -> Option<&Value> {
        todo!()
    }
}
