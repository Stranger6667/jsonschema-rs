use crate::{vocabularies::Validate, JsonSchema};

#[derive(Debug)]
pub struct Ref {}

impl Validate for Ref {
    fn is_valid(&self, _: &JsonSchema, _: &serde_json::Value) -> bool {
        todo!()
    }
}
