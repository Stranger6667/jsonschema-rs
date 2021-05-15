use crate::{
    compilation::context::CompilationContext,
    keywords::{exclusive_minimum, minimum, ValidationResult},
};
use serde_json::{Map, Value};

#[inline]
pub(crate) fn compile<'a>(
    parent: &'a Map<String, Value>,
    schema: &'a Value,
    context: &'a CompilationContext,
) -> Option<ValidationResult<'a>> {
    if let Some(Value::Bool(true)) = parent.get("exclusiveMinimum") {
        exclusive_minimum::compile(parent, schema, context)
    } else {
        minimum::compile(parent, schema, context)
    }
}
