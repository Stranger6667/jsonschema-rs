use crate::{vocabularies::Keyword, Schema};
use serde_json::Value;
use std::ops::Range;

#[derive(Debug, Eq, PartialEq)]
pub struct AllOf {
    pub(crate) edges: Range<usize>,
}

impl AllOf {
    pub(crate) fn build(start: usize, end: usize) -> Keyword {
        Self { edges: start..end }.into()
    }
}

impl AllOf {
    pub(crate) fn is_valid(&self, schema: &Schema, instance: &Value) -> bool {
        schema.edges[self.edges.clone()].iter().all(|edge| {
            schema.keywords[edge.keywords.clone()]
                .iter()
                .all(|k| k.is_valid(schema, instance))
        })
    }
}
