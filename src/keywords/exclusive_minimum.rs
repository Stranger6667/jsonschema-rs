use super::{CompilationResult, Validate};
use crate::{
    compilation::{CompilationContext, JSONSchema},
    error::{error, no_error, CompilationError, ErrorIterator, ValidationError},
};
use serde_json::{Map, Value};

pub struct ExclusiveMinimumValidator {
    limit: f64,
}

impl ExclusiveMinimumValidator {
    pub(crate) fn compile(schema: &Value) -> CompilationResult {
        if let Some(limit) = schema.as_f64() {
            return Ok(Box::new(ExclusiveMinimumValidator { limit }));
        }
        Err(CompilationError::SchemaError)
    }
}

impl Validate for ExclusiveMinimumValidator {
    fn validate<'a>(&self, _: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Some(item) = instance.as_f64() {
            if item <= self.limit {
                return error(ValidationError::exclusive_minimum(instance, self.limit));
            }
        }
        no_error()
    }

    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Some(item) = instance.as_f64() {
            if item <= self.limit {
                return false;
            }
        }
        true
    }

    fn name(&self) -> String {
        format!("<exclusive minimum: {}>", self.limit)
    }
}
pub(crate) fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    Some(ExclusiveMinimumValidator::compile(schema))
}
