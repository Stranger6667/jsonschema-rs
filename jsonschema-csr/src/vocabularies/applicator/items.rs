use crate::{vocabularies::Validate, JsonSchema};

#[derive(Debug)]
pub struct ItemsArray {}

impl Validate for ItemsArray {
    fn is_valid(&self, _: &JsonSchema, _: &serde_json::Value) -> bool {
        todo!()
    }
}
