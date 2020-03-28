use super::CompilationResult;
use super::Validate;
use crate::context::CompilationContext;
use crate::error::{no_error, CompilationError, ErrorIterator, ValidationError};
use crate::JSONSchema;
use serde_json::{Map, Value};

pub struct MaxLengthValidator {
    limit: usize,
}

impl MaxLengthValidator {
    pub(crate) fn compile(schema: &Value) -> CompilationResult {
        if let Value::Number(limit) = schema {
            let limit = limit.as_u64().unwrap() as usize;
            return Ok(Box::new(MaxLengthValidator { limit }));
        }
        Err(CompilationError::SchemaError)
    }
}

impl Validate for MaxLengthValidator {
    fn validate<'a>(&self, _schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::String(item) = instance {
            if item.chars().count() > self.limit {
                return ValidationError::max_length(item.clone());
            }
        }
        no_error()
    }
    fn name(&self) -> String {
        format!("<max length: {}>", self.limit)
    }
}
pub(crate) fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    Some(MaxLengthValidator::compile(schema))
}
