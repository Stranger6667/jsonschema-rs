use super::super::{exclusive_minimum, minimum, CompilationResult};
use crate::compilation::CompilationContext;
use serde_json::{Map, Value};

pub(crate) fn compile(
    parent: &Map<String, Value>,
    schema: &Value,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    if let Some(exclusive_maximum) = parent.get("exclusiveMinimum") {
        if let Some(value) = exclusive_maximum.as_bool() {
            if value {
                return exclusive_minimum::compile(parent, schema, context);
            }
        }
    }
    minimum::compile(parent, schema, context)
}
