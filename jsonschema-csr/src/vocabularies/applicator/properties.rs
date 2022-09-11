use crate::{
    compilation::edges::EdgeLabel,
    vocabularies::{Keyword, Validate},
    JsonSchema,
};
use serde_json::Value;

#[derive(Debug)]
// TODO: Maybe store start / end of child keywords
pub struct Properties {}

impl Properties {
    pub(crate) fn build() -> Keyword {
        Self {}.into()
    }
}

impl Validate for Properties {
    fn is_valid(&self, schema: &JsonSchema, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            schema.edges_of(1).all(|next| {
                if let Some(value) = match &next.label {
                    EdgeLabel::Key(key) => item.get(key),
                    EdgeLabel::Index(_) => unreachable!(),
                } {
                    schema
                        .edges_of(next.target - 1)
                        .all(|e| schema.keywords[e.target - 1].is_valid(schema, value))
                } else {
                    true
                }
            })
        } else {
            true
        }
    }
}
