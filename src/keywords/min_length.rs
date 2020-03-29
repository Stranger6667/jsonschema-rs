use super::CompilationResult;
use super::Validate;
use crate::compilation::CompilationContext;
use crate::error::{no_error, CompilationError, ErrorIterator, ValidationError};
use crate::JSONSchema;
use serde_json::{Map, Value};

pub struct MinLengthValidator {
    limit: usize,
}

impl<'a> MinLengthValidator {
    pub(crate) fn compile(schema: &Value) -> CompilationResult {
        if let Value::Number(limit) = schema {
            let limit = limit.as_u64().unwrap() as usize;
            return Ok(Box::new(MinLengthValidator { limit }));
        }
        Err(CompilationError::SchemaError)
    }
}

impl Validate for MinLengthValidator {
    fn validate<'a>(&self, _: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::String(item) = instance {
            if item.chars().count() < self.limit {
                return ValidationError::min_length(item.clone());
            }
        }
        no_error()
    }

    fn name(&self) -> String {
        format!("<min length: {}>", self.limit)
    }
}
pub(crate) fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    Some(MinLengthValidator::compile(schema))
}
