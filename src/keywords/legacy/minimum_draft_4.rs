/// Docs: <https://tools.ietf.org/html/draft-fge-json-schema-validation-00#section-5.1.3>
use crate::{
    compilation::CompilationContext,
    keywords::{exclusive_minimum, minimum, CompilationResult},
};
use serde_json::{Map, Value};

#[inline]
pub fn compile(
    parent: &Map<String, Value>,
    schema: &Value,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    // The value of "minimum" MUST be a JSON number.
    // The value of "exclusiveMinimum" MUST be a boolean.
    if let Some(Value::Bool(true)) = parent.get("exclusiveMinimum") {
        exclusive_minimum::compile(parent, schema, context)
    } else {
        // "exclusiveMinimum", if absent, may be considered as being present with boolean value false
        minimum::compile(parent, schema, context)
    }
}
