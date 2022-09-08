use crate::vocabularies::{Keyword, Validate};

#[derive(Debug)]
pub struct Properties {}

impl Properties {
    pub(crate) fn build() -> Keyword {
        Self {}.into()
    }
}

impl Validate for Properties {
    fn is_valid(&self, _: &serde_json::Value) -> bool {
        todo!()
    }
}
