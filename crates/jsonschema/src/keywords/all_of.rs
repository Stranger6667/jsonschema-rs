use crate::{
    compiler,
    error::{ErrorIterator, ValidationError},
    node::SchemaNode,
    output::BasicOutput,
    paths::{JsonPointer, JsonPointerNode},
    primitive_type::PrimitiveType,
    validator::{PartialApplication, Validate},
};
use serde_json::{Map, Value};

use super::CompilationResult;

pub(crate) struct AllOfValidator {
    schemas: Vec<SchemaNode>,
}

impl AllOfValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        ctx: &compiler::Context,
        items: &'a [Value],
    ) -> CompilationResult<'a> {
        let ctx = ctx.with_path("allOf");
        let mut schemas = Vec::with_capacity(items.len());
        for (idx, item) in items.iter().enumerate() {
            let ctx = ctx.with_path(idx);
            let validators = compiler::compile(&ctx, ctx.as_resource_ref(item))?;
            schemas.push(validators)
        }
        Ok(Box::new(AllOfValidator { schemas }))
    }
}

impl Validate for AllOfValidator {
    fn is_valid(&self, instance: &Value) -> bool {
        self.schemas.iter().all(|n| n.is_valid(instance))
    }

    #[allow(clippy::needless_collect)]
    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &JsonPointerNode,
    ) -> ErrorIterator<'instance> {
        let errors: Vec<_> = self
            .schemas
            .iter()
            .flat_map(move |node| node.validate(instance, instance_path))
            .collect();
        Box::new(errors.into_iter())
    }

    fn apply<'a>(
        &'a self,
        instance: &Value,
        instance_path: &JsonPointerNode,
    ) -> PartialApplication<'a> {
        self.schemas
            .iter()
            .map(move |node| node.apply_rooted(instance, instance_path))
            .sum::<BasicOutput<'_>>()
            .into()
    }
}

pub(crate) struct SingleValueAllOfValidator {
    node: SchemaNode,
}

impl SingleValueAllOfValidator {
    #[inline]
    pub(crate) fn compile<'a>(ctx: &compiler::Context, schema: &'a Value) -> CompilationResult<'a> {
        let ctx = ctx.with_path("allOf");
        let ctx = ctx.with_path(0);
        let node = compiler::compile(&ctx, ctx.as_resource_ref(schema))?;
        Ok(Box::new(SingleValueAllOfValidator { node }))
    }
}

impl Validate for SingleValueAllOfValidator {
    fn is_valid(&self, instance: &Value) -> bool {
        self.node.is_valid(instance)
    }

    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &JsonPointerNode,
    ) -> ErrorIterator<'instance> {
        self.node.validate(instance, instance_path)
    }

    fn apply<'a>(
        &'a self,
        instance: &Value,
        instance_path: &JsonPointerNode,
    ) -> PartialApplication<'a> {
        self.node.apply_rooted(instance, instance_path).into()
    }
}

#[inline]
pub(crate) fn compile<'a>(
    ctx: &compiler::Context,
    _: &'a Map<String, Value>,
    schema: &'a Value,
) -> Option<CompilationResult<'a>> {
    if let Value::Array(items) = schema {
        if items.len() == 1 {
            let value = items.iter().next().expect("Vec is not empty");
            Some(SingleValueAllOfValidator::compile(ctx, value))
        } else {
            Some(AllOfValidator::compile(ctx, items))
        }
    } else {
        Some(Err(ValidationError::single_type_error(
            JsonPointer::default(),
            ctx.clone().into_pointer(),
            schema,
            PrimitiveType::Array,
        )))
    }
}

#[cfg(test)]
mod tests {
    use crate::tests_util;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(&json!({"allOf": [{"type": "string"}]}), &json!(1), "/allOf/0/type")]
    #[test_case(&json!({"allOf": [{"type": "integer"}, {"maximum": 5}]}), &json!(6), "/allOf/1/maximum")]
    fn schema_path(schema: &Value, instance: &Value, expected: &str) {
        tests_util::assert_schema_path(schema, instance, expected)
    }
}
