use super::{CompilationResult, Validate};
use crate::compilation::{CompilationContext, JSONSchema};
use crate::error::{no_error, CompilationError, ErrorIterator, ValidationError};
use serde_json::{Map, Value};

pub struct MaximumValidator {
    limit: f64,
}

impl MaximumValidator {
    pub(crate) fn compile(schema: &Value) -> CompilationResult {
        if let Value::Number(limit) = schema {
            let limit = limit.as_f64().unwrap();
            return Ok(Box::new(MaximumValidator { limit }));
        }
        Err(CompilationError::SchemaError)
    }
}

impl Validate for MaximumValidator {
    fn validate<'a>(&self, _: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Number(item) = instance {
            let item = item.as_f64().unwrap();
            if item > self.limit {
                return ValidationError::maximum(item, self.limit);
            }
        }
        no_error()
    }
    fn name(&self) -> String {
        format!("<maximum: {}>", self.limit)
    }
}

pub(crate) fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    Some(MaximumValidator::compile(schema))
}
