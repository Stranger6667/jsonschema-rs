use crate::{
    compiler,
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    node::SchemaNode,
    paths::{JsonPointer, JsonPointerNode},
    validator::Validate,
};
use serde_json::{Map, Value};

pub(crate) struct NotValidator {
    // needed only for error representation
    original: Value,
    node: SchemaNode,
    schema_path: JsonPointer,
}

impl NotValidator {
    #[inline]
    pub(crate) fn compile<'a>(ctx: &compiler::Context, schema: &'a Value) -> CompilationResult<'a> {
        let ctx = ctx.with_path("not");
        Ok(Box::new(NotValidator {
            original: schema.clone(),
            node: compiler::compile(&ctx, ctx.as_resource_ref(schema))?,
            schema_path: ctx.into_pointer(),
        }))
    }
}

impl Validate for NotValidator {
    fn is_valid(&self, instance: &Value) -> bool {
        !self.node.is_valid(instance)
    }

    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &JsonPointerNode,
    ) -> ErrorIterator<'instance> {
        if self.is_valid(instance) {
            no_error()
        } else {
            error(ValidationError::not(
                self.schema_path.clone(),
                instance_path.into(),
                instance,
                self.original.clone(),
            ))
        }
    }
}

#[inline]
pub(crate) fn compile<'a>(
    ctx: &compiler::Context,
    _: &'a Map<String, Value>,
    schema: &'a Value,
) -> Option<CompilationResult<'a>> {
    Some(NotValidator::compile(ctx, schema))
}

#[cfg(test)]
mod tests {
    use crate::tests_util;
    use serde_json::json;

    #[test]
    fn schema_path() {
        tests_util::assert_schema_path(&json!({"not": {"type": "string"}}), &json!("foo"), "/not")
    }
}
