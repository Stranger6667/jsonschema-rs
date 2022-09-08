use crate::vocabularies::Validate;

#[derive(Debug)]
pub struct Properties {}

impl Validate for Properties {
    fn is_valid(&self, _: &serde_json::Value) -> bool {
        todo!()
    }
}
