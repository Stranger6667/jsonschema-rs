use crate::{
    compilation::context::CompilationContext,
    keywords::{exclusive_maximum, maximum, ValidationResult},
};
use serde_json::{Map, Value};

#[inline]
pub(crate) fn compile<'a>(
    parent: &'a Map<String, Value>,
    schema: &'a Value,
    context: &'a CompilationContext,
) -> Option<ValidationResult<'a>> {
    if let Some(Value::Bool(true)) = parent.get("exclusiveMaximum") {
        exclusive_maximum::compile(parent, schema, context)
    } else {
        maximum::compile(parent, schema, context)
    }
}
