use crate::compilation::ValidatorArena;
use crate::{
    compilation::context::CompilationContext,
    keywords::{exclusive_maximum, maximum, CompilationResult},
};
use serde_json::{Map, Value};

#[inline]
pub(crate) fn compile<'a>(
    parent: &'a Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
    arena: &mut ValidatorArena,
) -> Option<CompilationResult<'a>> {
    if let Some(Value::Bool(true)) = parent.get("exclusiveMaximum") {
        exclusive_maximum::compile(parent, schema, context, arena)
    } else {
        maximum::compile(parent, schema, context, arena)
    }
}
