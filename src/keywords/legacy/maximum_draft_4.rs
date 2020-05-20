/// Docs: https://tools.ietf.org/html/draft-fge-json-schema-validation-00#section-5.1.2
use crate::{
    compilation::CompilationContext,
    keywords::{exclusive_maximum, maximum, CompilationResult},
};
use serde_json::{Map, Value};

#[inline]
pub fn compile(
    parent: &Map<String, Value>,
    schema: &Value,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    // The value of "maximum" MUST be a JSON number.
    // The value of "exclusiveMaximum" MUST be a boolean.
    if let Some(Value::Bool(true)) = parent.get("exclusiveMaximum") {
        exclusive_maximum::compile(parent, schema, context)
    } else {
        // "exclusiveMaximum", if absent, may be considered as being present with boolean value false
        maximum::compile(parent, schema, context)
    }
}
