use crate::vocabularies::Validate;

#[derive(Debug)]
pub struct ItemsArray {}

impl Validate for ItemsArray {
    fn is_valid(&self, _: &serde_json::Value) -> bool {
        todo!()
    }
}
