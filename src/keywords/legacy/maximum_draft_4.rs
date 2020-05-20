use super::super::{exclusive_maximum, maximum, CompilationResult};
use crate::compilation::CompilationContext;
use serde_json::{Map, Value};

#[inline]
pub fn compile(
    parent: &Map<String, Value>,
    schema: &Value,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    if let Some(Value::Bool(true)) = parent.get("exclusiveMaximum") {
        exclusive_maximum::compile(parent, schema, context)
    } else {
        maximum::compile(parent, schema, context)
    }
}
