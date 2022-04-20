use super::datatype::Value;

pub trait Source {
    fn get_tpl_const(&self, key: &str) -> Option<&Value>;
}
