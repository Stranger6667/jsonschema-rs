use crate::vocabularies::{Keyword, Validate};

#[derive(Debug)]
pub struct ItemsArray {}

impl Validate for ItemsArray {
    fn is_valid(&self, _: &[Keyword], _: &serde_json::Value) -> bool {
        todo!()
    }
}
