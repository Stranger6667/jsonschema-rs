use super::super::{exclusive_maximum, maximum, CompilationResult};
use crate::compilation::CompilationContext;
use serde_json::{Map, Value};

#[inline]
pub fn compile(
    parent: &Map<String, Value>,
    schema: &Value,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    match parent.get("exclusiveMaximum") {
        Some(Value::Bool(true)) => exclusive_maximum::compile(parent, schema, context),
        _ => maximum::compile(parent, schema, context),
    }
}
