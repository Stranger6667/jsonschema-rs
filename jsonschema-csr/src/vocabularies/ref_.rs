use crate::vocabularies::{Keyword, Validate};

#[derive(Debug)]
pub struct Ref {}

impl Validate for Ref {
    fn is_valid(&self, _: &[Keyword], _: &serde_json::Value) -> bool {
        todo!()
    }
}
