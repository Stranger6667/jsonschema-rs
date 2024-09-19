use crate::{
    compiler,
    error::{error, no_error, ErrorIterator, ValidationError},
    node::SchemaNode,
    paths::JsonPointerNode,
    primitive_type::PrimitiveType,
    validator::{PartialApplication, Validate},
};
use serde_json::{Map, Value};

use super::CompilationResult;
use crate::paths::JsonPointer;

pub(crate) struct AnyOfValidator {
    schemas: Vec<SchemaNode>,
    schema_path: JsonPointer,
}

impl AnyOfValidator {
    #[inline]
    pub(crate) fn compile<'a>(ctx: &compiler::Context, schema: &'a Value) -> CompilationResult<'a> {
        if let Value::Array(items) = schema {
            let ctx = ctx.with_path("anyOf");
            let mut schemas = Vec::with_capacity(items.len());
            for (idx, item) in items.iter().enumerate() {
                let ctx = ctx.with_path(idx);
                let node = compiler::compile(&ctx, ctx.as_resource_ref(item))?;
                schemas.push(node)
            }
            Ok(Box::new(AnyOfValidator {
                schemas,
                schema_path: ctx.into_pointer(),
            }))
        } else {
            Err(ValidationError::single_type_error(
                JsonPointer::default(),
                ctx.clone().into_pointer(),
                schema,
                PrimitiveType::Array,
            ))
        }
    }
}

impl Validate for AnyOfValidator {
    fn is_valid(&self, instance: &Value) -> bool {
        self.schemas.iter().any(|s| s.is_valid(instance))
    }

    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &JsonPointerNode,
    ) -> ErrorIterator<'instance> {
        if self.is_valid(instance) {
            no_error()
        } else {
            error(ValidationError::any_of(
                self.schema_path.clone(),
                instance_path.into(),
                instance,
            ))
        }
    }

    fn apply<'a>(
        &'a self,
        instance: &Value,
        instance_path: &JsonPointerNode,
    ) -> PartialApplication<'a> {
        let mut successes = Vec::new();
        let mut failures = Vec::new();
        for node in &self.schemas {
            let result = node.apply_rooted(instance, instance_path);
            if result.is_valid() {
                successes.push(result);
            } else {
                failures.push(result);
            }
        }
        if successes.is_empty() {
            failures.into_iter().collect()
        } else {
            successes.into_iter().collect()
        }
    }
}

#[inline]
pub(crate) fn compile<'a>(
    ctx: &compiler::Context,
    _: &'a Map<String, Value>,
    schema: &'a Value,
) -> Option<CompilationResult<'a>> {
    Some(AnyOfValidator::compile(ctx, schema))
}

#[cfg(test)]
mod tests {
    use crate::tests_util;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(&json!({"anyOf": [{"type": "string"}]}), &json!(1), "/anyOf")]
    #[test_case(&json!({"anyOf": [{"type": "integer"}, {"type": "string"}]}), &json!({}), "/anyOf")]
    fn schema_path(schema: &Value, instance: &Value, expected: &str) {
        tests_util::assert_schema_path(schema, instance, expected)
    }
}
