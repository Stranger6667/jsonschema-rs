use crate::vocabularies::Validate;

#[derive(Debug)]
pub struct Ref {}

impl Validate for Ref {
    fn is_valid(&self, _: &serde_json::Value) -> bool {
        todo!()
    }
}
