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
            // TODO. edges are known upfront - no need to calculate offsets
            schema.edges[1..2].iter().all(|next| {
                if let Some(value) = match &next.label {
                    EdgeLabel::Key(key) => item.get(key),
                    EdgeLabel::Index(_) => unreachable!(),
                } {
                    // TODO: The keyword range is also known upfront for each edge - store it there
                    schema.keywords[1..2]
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
