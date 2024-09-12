use crate::{
    compilation::{compile_validators, context::CompilationContext},
    error::{ErrorIterator, ValidationError},
    output::BasicOutput,
    paths::{JSONPointer, JsonPointerNode},
    primitive_type::PrimitiveType,
    schema_node::SchemaNode,
    validator::{format_iter_of_validators, format_validators, PartialApplication, Validate},
};
use serde_json::{Map, Value};

use super::CompilationResult;

pub(crate) struct AllOfValidator {
    schemas: Vec<SchemaNode>,
}

impl AllOfValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        items: &'a [Value],
        context: &CompilationContext,
    ) -> CompilationResult<'a> {
        let keyword_context = context.with_path("allOf");
        let mut schemas = Vec::with_capacity(items.len());
        for (idx, item) in items.iter().enumerate() {
            let item_context = keyword_context.with_path(idx);
            let validators = compile_validators(item, &item_context)?;
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

impl core::fmt::Display for AllOfValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "allOf: [{}]",
            format_iter_of_validators(self.schemas.iter().map(SchemaNode::validators))
        )
    }
}
pub(crate) struct SingleValueAllOfValidator {
    node: SchemaNode,
}

impl SingleValueAllOfValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        schema: &'a Value,
        context: &CompilationContext,
    ) -> CompilationResult<'a> {
        let keyword_context = context.with_path("allOf");
        let item_context = keyword_context.with_path(0);
        let node = compile_validators(schema, &item_context)?;
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

impl core::fmt::Display for SingleValueAllOfValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "allOf: [{}]", format_validators(self.node.validators()))
    }
}
#[inline]
pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    if let Value::Array(items) = schema {
        if items.len() == 1 {
            let value = items.iter().next().expect("Vec is not empty");
            Some(SingleValueAllOfValidator::compile(value, context))
        } else {
            Some(AllOfValidator::compile(items, context))
        }
    } else {
        Some(Err(ValidationError::single_type_error(
            JSONPointer::default(),
            context.clone().into_pointer(),
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
