use crate::{compilation::edges::EdgeLabel, vocabularies::Keyword, JsonSchema};
use serde_json::Value;
use std::ops::Range;

#[derive(Debug, Eq, PartialEq)]
pub struct Properties {
    edges: Range<usize>,
}

impl Properties {
    pub(crate) fn build(start: usize, end: usize) -> Keyword {
        Self { edges: start..end }.into()
    }
}

impl Properties {
    pub(crate) fn is_valid(&self, schema: &JsonSchema, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            schema.edges[self.edges.clone()].iter().all(|edge| {
                if let Some(value) = match &edge.label {
                    EdgeLabel::Key(key) => item.get(key),
                    EdgeLabel::Index(_) | EdgeLabel::Keyword(_) => unreachable!(),
                } {
                    schema.keywords[edge.keywords.clone()]
                        .iter()
                        .all(|k| k.is_valid(schema, value))
                } else {
                    true
                }
            })
        } else {
            true
        }
    }
}
