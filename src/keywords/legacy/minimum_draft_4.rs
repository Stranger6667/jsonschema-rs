use super::super::{exclusive_minimum, minimum, CompilationResult};
use crate::compilation::CompilationContext;
use serde_json::{Map, Value};

#[inline]
pub(crate) fn compile(
    parent: &Map<String, Value>,
    schema: &Value,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    match parent.get("exclusiveMinimum") {
        Some(Value::Bool(true)) => exclusive_minimum::compile(parent, schema, context),
        _ => minimum::compile(parent, schema, context),
    }
}
