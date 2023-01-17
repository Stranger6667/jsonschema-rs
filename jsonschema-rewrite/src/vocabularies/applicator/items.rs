use crate::Schema;

#[derive(Debug, Eq, PartialEq)]
pub struct ItemsArray {}

impl ItemsArray {
    pub(crate) fn is_valid(&self, _: &Schema, _: &serde_json::Value) -> bool {
        todo!()
    }
}
