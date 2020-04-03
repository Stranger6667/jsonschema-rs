use super::super::CompilationResult;
use super::super::{exclusive_maximum, maximum};
use crate::compilation::CompilationContext;
use serde_json::{Map, Value};

pub(crate) fn compile(
    parent: &Map<String, Value>,
    schema: &Value,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    match parent.get("exclusiveMaximum") {
        Some(Value::Bool(true)) => exclusive_maximum::compile(parent, schema, context),
        _ => maximum::compile(parent, schema, context),
    }
}
