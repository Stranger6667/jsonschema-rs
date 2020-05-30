use crate::{
    compilation::{CompilationContext, JSONSchema},
    error::{CompilationError, ValidationError},
    keywords::CompilationResult,
    validator::Validate,
};
use serde_json::{Map, Value};

pub struct MinLengthValidator {
    limit: u64,
}

impl MinLengthValidator {
    #[inline]
    pub(crate) fn compile(schema: &Value) -> CompilationResult {
        if let Some(limit) = schema.as_u64() {
            return Ok(Box::new(MinLengthValidator { limit }));
        }
        Err(CompilationError::SchemaError)
    }
}

impl Validate for MinLengthValidator {
    #[inline]
    fn build_validation_error<'a>(&self, instance: &'a Value) -> ValidationError<'a> {
        ValidationError::min_length(instance, self.limit)
    }

    fn name(&self) -> String {
        format!("minLength: {}", self.limit)
    }

    #[inline]
    fn is_valid_string(&self, _: &JSONSchema, _: &Value, instance_value: &str) -> bool {
        instance_value.chars().count() as u64 >= self.limit
    }
}

#[inline]
pub fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    Some(MinLengthValidator::compile(schema))
}
