use crate::{
    compilation::CompilationContext,
    keywords::{exclusive_minimum, minimum, CompilationResult},
};
use serde_json::{Map, Value};

#[inline]
pub(crate) fn compile(
    parent: &Map<String, Value>,
    schema: &Value,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    if let Some(Value::Bool(true)) = parent.get("exclusiveMinimum") {
        exclusive_minimum::compile(parent, schema, context)
    } else {
        minimum::compile(parent, schema, context)
    }
}
