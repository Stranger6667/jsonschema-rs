use crate::{vocabularies::Keyword, Schema};
use serde_json::Value;
use std::ops::Range;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Ref {
    pub(crate) nodes: Range<usize>,
}

impl Ref {
    pub(crate) fn build(nodes: Range<usize>) -> Keyword {
        Self { nodes }.into()
    }
}

impl Ref {
    pub(crate) fn is_valid(&self, schema: &Schema, instance: &Value) -> bool {
        schema.nodes()[self.nodes.clone()]
            .iter()
            .all(|k| k.is_valid(schema, instance))
    }
}
