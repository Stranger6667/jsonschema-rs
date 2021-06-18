use crate::{
    compilation::context::CompilationContext,
    keywords::{exclusive_minimum, minimum, CompilationResult},
};
use serde_json::{Map, Value};

#[inline]
pub(crate) fn compile<'a>(
    parent: &'a Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    if let Some(Value::Bool(true)) = parent.get("exclusiveMinimum") {
        exclusive_minimum::compile(parent, schema, context)
    } else {
        minimum::compile(parent, schema, context)
    }
}
