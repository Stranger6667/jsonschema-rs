use crate::{
    compilation::{context::CompilationContext, JSONSchema},
    error::{no_error, CompilationError, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    validator::Validate,
};
use serde_json::{Map, Value};

pub(crate) struct MaxLengthValidator {
    limit: u64,
}

impl MaxLengthValidator {
    #[inline]
    pub(crate) fn compile(schema: &Value) -> CompilationResult {
        if let Some(limit) = schema.as_u64() {
            return Ok(Box::new(MaxLengthValidator { limit }));
        }
        Err(CompilationError::SchemaError)
    }
}

impl Validate for MaxLengthValidator {
    #[inline]
    fn build_validation_error<'a>(&self, instance: &'a Value) -> ValidationError<'a> {
        ValidationError::max_length(instance, self.limit)
    }

    #[inline]
    fn is_valid_string(&self, _: &JSONSchema, _: &Value, instance_value: &str) -> bool {
        instance_value.chars().count() as u64 <= self.limit
    }
    #[inline]
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(instance_value) = instance {
            self.is_valid_string(schema, instance, instance_value)
        } else {
            true
        }
    }

    #[inline]
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::String(instance_value) = instance {
            self.validate_string(schema, instance, instance_value)
        } else {
            no_error()
        }
    }
}
impl ToString for MaxLengthValidator {
    fn to_string(&self) -> String {
        format!("maxLength: {}", self.limit)
    }
}

#[inline]
pub(crate) fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    Some(MaxLengthValidator::compile(schema))
}
