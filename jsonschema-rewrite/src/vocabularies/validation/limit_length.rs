use crate::vocabularies::Keyword;
use serde_json::Value;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct MaxLength {
    pub(crate) limit: u64,
}

impl MaxLength {
    pub(crate) fn build(limit: u64) -> Keyword {
        Self { limit }.into()
    }
}

impl MaxLength {
    pub(crate) fn is_valid(&self, _: &Value) -> bool {
        true
    }
}
