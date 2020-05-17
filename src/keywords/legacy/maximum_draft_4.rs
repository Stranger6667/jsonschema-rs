use super::super::{exclusive_maximum, maximum, CompilationResult};
use crate::compilation::CompilationContext;
use serde_json::{Map, Value};

pub(crate) fn compile(
    parent: &Map<String, Value>,
    schema: &Value,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    if let Some(exclusive_maximum) = parent.get("exclusiveMaximum") {
        if let Some(value) = exclusive_maximum.as_bool() {
            if value {
                return exclusive_maximum::compile(parent, schema, context);
            }
        }
    }
    maximum::compile(parent, schema, context)
}
