use crate::vocabularies::{Validate, Vocabulary};
use serde_json::Value;

#[derive(Debug)]
pub struct Properties {}

impl Validate for Properties {
    fn vocabulary(&self) -> Vocabulary {
        Vocabulary::Applicator
    }
    fn is_valid(&self, _: &Value) -> bool {
        todo!()
    }
}
