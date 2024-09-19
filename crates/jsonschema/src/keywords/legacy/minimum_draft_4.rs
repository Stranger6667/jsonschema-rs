use crate::{
    compiler,
    keywords::{exclusive_minimum, minimum, CompilationResult},
};
use serde_json::{Map, Value};

#[inline]
pub(crate) fn compile<'a>(
    ctx: &compiler::Context,
    parent: &'a Map<String, Value>,
    schema: &'a Value,
) -> Option<CompilationResult<'a>> {
    if let Some(Value::Bool(true)) = parent.get("exclusiveMinimum") {
        exclusive_minimum::compile(ctx, parent, schema)
    } else {
        minimum::compile(ctx, parent, schema)
    }
}
