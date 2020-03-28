use super::CompilationResult;
use super::Validate;
use crate::context::CompilationContext;
use crate::error::{no_error, CompilationError, ErrorIterator, ValidationError};
use crate::JSONSchema;
use serde_json::{Map, Value};

pub struct ExclusiveMaximumValidator {
    limit: f64,
}

impl<'a> ExclusiveMaximumValidator {
    pub(crate) fn compile(schema: &Value) -> CompilationResult {
        if let Value::Number(limit) = schema {
            return Ok(Box::new(ExclusiveMaximumValidator {
                limit: limit.as_f64().unwrap(),
            }));
        }
        Err(CompilationError::SchemaError)
    }
}

impl Validate for ExclusiveMaximumValidator {
    fn validate<'a>(&self, _: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Number(item) = instance {
            let item = item.as_f64().unwrap();
            if item >= self.limit {
                return ValidationError::exclusive_maximum(item, self.limit);
            }
        }
        no_error()
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
