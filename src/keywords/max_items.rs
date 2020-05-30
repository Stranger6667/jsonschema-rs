use crate::{
    compilation::{CompilationContext, JSONSchema},
    error::{CompilationError, ValidationError},
    keywords::CompilationResult,
    validator::Validate,
};
use serde_json::{Map, Value};

pub struct MaxItemsValidator {
    limit: u64,
}

impl MaxItemsValidator {
    #[inline]
    pub(crate) fn compile(schema: &Value) -> CompilationResult {
        if let Some(limit) = schema.as_u64() {
            return Ok(Box::new(MaxItemsValidator { limit }));
        }
        Err(CompilationError::SchemaError)
    }
}

impl Validate for MaxItemsValidator {
    #[inline]
    fn build_validation_error<'a>(&self, instance: &'a Value) -> ValidationError<'a> {
        ValidationError::max_items(instance, self.limit)
    }

    fn name(&self) -> String {
        format!("maxItems: {}", self.limit)
    }

    #[inline]
    fn is_valid_array(&self, _: &JSONSchema, _: &Value, instance_value: &[Value]) -> bool {
        instance_value.len() as u64 <= self.limit
    }
}

#[inline]
pub fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    Some(MaxItemsValidator::compile(schema))
}
