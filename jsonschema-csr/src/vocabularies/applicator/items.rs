use crate::vocabularies::{Validate, Vocabulary};

#[derive(Debug)]
pub struct ItemsArray {}

impl Validate for ItemsArray {
    fn vocabulary(&self) -> Vocabulary {
        Vocabulary::Applicator
    }
    fn is_valid(&self, _: &serde_json::Value) -> bool {
        todo!()
    }
}
