use crate::{
    compiler,
    error::{ErrorIterator, ValidationError},
    node::SchemaNode,
    output::BasicOutput,
    paths::{LazyLocation, Location},
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
        let ctx = ctx.new_at_location("allOf");
        let mut schemas = Vec::with_capacity(items.len());
        for (idx, item) in items.iter().enumerate() {
            let ctx = ctx.new_at_location(idx);
            let validators = compiler::compile(&ctx, ctx.as_resource_ref(item))?;
            schemas.push(validators)
        }
        Ok(Box::new(AllOfValidator { schemas }))
    }
}

impl Validate for AllOfValidator {
    #[allow(clippy::needless_collect)]
    fn iter_errors<'i>(&self, instance: &'i Value, location: &LazyLocation) -> ErrorIterator<'i> {
        let errors: Vec<_> = self
            .schemas
            .iter()
            .flat_map(move |node| node.iter_errors(instance, location))
            .collect();
        Box::new(errors.into_iter())
    }

    fn is_valid(&self, instance: &Value) -> bool {
        self.schemas.iter().all(|n| n.is_valid(instance))
    }

    fn validate<'i>(
        &self,
        instance: &'i Value,
        location: &LazyLocation,
    ) -> Result<(), ValidationError<'i>> {
        for schema in &self.schemas {
            schema.validate(instance, location)?;
        }
        Ok(())
    }

    fn apply<'a>(&'a self, instance: &Value, location: &LazyLocation) -> PartialApplication<'a> {
        self.schemas
            .iter()
            .map(move |node| node.apply_rooted(instance, location))
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
        let ctx = ctx.new_at_location("allOf");
        let ctx = ctx.new_at_location(0);
        let node = compiler::compile(&ctx, ctx.as_resource_ref(schema))?;
        Ok(Box::new(SingleValueAllOfValidator { node }))
    }
}

impl Validate for SingleValueAllOfValidator {
    fn iter_errors<'i>(&self, instance: &'i Value, location: &LazyLocation) -> ErrorIterator<'i> {
        self.node.iter_errors(instance, location)
    }

    fn is_valid(&self, instance: &Value) -> bool {
        self.node.is_valid(instance)
    }

    fn validate<'i>(
        &self,
        instance: &'i Value,
        location: &LazyLocation,
    ) -> Result<(), ValidationError<'i>> {
        self.node.validate(instance, location)
    }

    fn apply<'a>(&'a self, instance: &Value, location: &LazyLocation) -> PartialApplication<'a> {
        self.node.apply_rooted(instance, location).into()
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
            Location::new(),
            ctx.location().clone(),
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
    fn location(schema: &Value, instance: &Value, expected: &str) {
        tests_util::assert_schema_location(schema, instance, expected)
    }
}
