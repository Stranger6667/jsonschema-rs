use crate::JsonSchema;

#[derive(Debug, Eq, PartialEq)]
pub struct ItemsArray {}

impl ItemsArray {
    pub(crate) fn is_valid(&self, _: &JsonSchema, _: &serde_json::Value) -> bool {
        todo!()
    }
}
