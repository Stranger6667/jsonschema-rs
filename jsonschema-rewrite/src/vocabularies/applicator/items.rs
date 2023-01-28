use crate::{vocabularies::Keyword, Schema};
use serde_json::Value;

#[derive(Debug, Eq, PartialEq)]
pub struct Items {}

impl Items {
    pub(crate) fn build() -> Keyword {
        Self {}.into()
    }
}

impl Items {
    pub(crate) fn is_valid(&self, _: &Schema, _: &Value) -> bool {
        todo!()
    }
}
