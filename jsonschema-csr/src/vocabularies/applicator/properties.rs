use crate::vocabularies::{Keyword, Validate};
use serde_json::Value;

#[derive(Debug)]
pub struct Properties {}

impl Validate for Properties {
    fn is_valid(&self, _: &[Keyword], _: &Value) -> bool {
        todo!()
    }
}
