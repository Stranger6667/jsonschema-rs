use super::{CompilationResult, Validate};
use crate::{
    compilation::{CompilationContext, JSONSchema},
    error::{error, no_error, CompilationError, ErrorIterator, ValidationError},
};
use serde_json::{Map, Value};

pub struct ExclusiveMaximumValidator {
    limit: f64,
}

impl<'a> ExclusiveMaximumValidator {
    pub(crate) fn compile(schema: &Value) -> CompilationResult {
        if let Some(limit) = schema.as_f64() {
            Ok(Box::new(ExclusiveMaximumValidator { limit }))
        } else {
            Err(CompilationError::SchemaError)
        }
    }
}

impl Validate for ExclusiveMaximumValidator {
    fn validate<'a>(&self, _: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Some(item) = instance.as_f64() {
            if item >= self.limit {
                return error(ValidationError::exclusive_maximum(instance, self.limit));
            }
        }
        no_error()
    }

    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Some(item) = instance.as_f64() {
            if item >= self.limit {
                return false;
            }
        }
        true
    }

    fn name(&self) -> String {
        format!("<exclusive maximum: {}>", self.limit)
    }
}

pub(crate) fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    Some(ExclusiveMaximumValidator::compile(schema))
}
