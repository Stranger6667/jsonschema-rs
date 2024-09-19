use crate::{
    compiler,
    keywords::{exclusive_maximum, maximum, CompilationResult},
};
use serde_json::{Map, Value};

#[inline]
pub(crate) fn compile<'a>(
    ctx: &compiler::Context,
    parent: &'a Map<String, Value>,
    schema: &'a Value,
) -> Option<CompilationResult<'a>> {
    if let Some(Value::Bool(true)) = parent.get("exclusiveMaximum") {
        exclusive_maximum::compile(ctx, parent, schema)
    } else {
        maximum::compile(ctx, parent, schema)
    }
}
