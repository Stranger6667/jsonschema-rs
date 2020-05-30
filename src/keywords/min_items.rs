use crate::{
    compilation::{CompilationContext, JSONSchema},
    error::{CompilationError, ValidationError},
    keywords::CompilationResult,
    validator::Validate,
};
use serde_json::{Map, Value};

pub struct MinItemsValidator {
    limit: u64,
}

impl MinItemsValidator {
    #[inline]
    pub(crate) fn compile(schema: &Value) -> CompilationResult {
        if let Some(limit) = schema.as_u64() {
            return Ok(Box::new(MinItemsValidator { limit }));
        }
        Err(CompilationError::SchemaError)
    }
}

impl Validate for MinItemsValidator {
    #[inline]
    fn build_validation_error<'a>(&self, instance: &'a Value) -> ValidationError<'a> {
        ValidationError::min_items(instance, self.limit)
    }

    fn name(&self) -> String {
        format!("minItems: {}", self.limit)
    }

    #[inline]
    fn is_valid_array(&self, _: &JSONSchema, _: &Value, instance_value: &[Value]) -> bool {
        instance_value.len() as u64 >= self.limit
    }
}

#[inline]
pub fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    Some(MinItemsValidator::compile(schema))
}
