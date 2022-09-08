use crate::vocabularies::{Validate, Vocabulary};

#[derive(Debug)]
pub struct Ref {}

impl Validate for Ref {
    fn vocabulary(&self) -> Vocabulary {
        Vocabulary::Core
    }
    fn is_valid(&self, _: &serde_json::Value) -> bool {
        todo!()
    }
}
