use crate::{compilation::edges::EdgeLabel, vocabularies::Keyword, JsonSchema};
use serde_json::Value;
use std::ops::Range;

#[derive(Debug, Eq, PartialEq)]
pub struct AllOf {
    keywords: Range<usize>,
}

impl AllOf {
    pub(crate) fn build(start: usize, end: usize) -> Keyword {
        Self {
            keywords: start..end,
        }
        .into()
    }
}

impl AllOf {
    pub(crate) fn is_valid(&self, schema: &JsonSchema, instance: &Value) -> bool {
        schema.keywords[self.keywords.clone()]
            .iter()
            .all(|k| k.is_valid(schema, instance))
    }
}
