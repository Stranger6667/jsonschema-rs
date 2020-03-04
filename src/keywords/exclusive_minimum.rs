use super::Validate;
use super::{CompilationResult, ValidationResult};
use crate::context::CompilationContext;
use crate::error::{CompilationError, ValidationError};
use crate::JSONSchema;
use serde_json::{Map, Value};

pub struct ExclusiveMinimumValidator {
    limit: f64,
}

impl<'a> ExclusiveMinimumValidator {
    pub(crate) fn compile(schema: &Value) -> CompilationResult<'a> {
        if let Value::Number(limit) = schema {
            let limit = limit.as_f64().unwrap();
            return Ok(Box::new(ExclusiveMinimumValidator { limit }));
        }
        Err(CompilationError::SchemaError)
    }
}

impl<'a> Validate<'a> for ExclusiveMinimumValidator {
    fn validate(&self, _: &JSONSchema, instance: &Value) -> ValidationResult {
        if let Value::Number(item) = instance {
            let item = item.as_f64().unwrap();
            if item <= self.limit {
                return Err(ValidationError::exclusive_minimum(item, self.limit));
            }
        }
        Ok(())
    }
    fn name(&self) -> String {
        format!("<exclusive minimum: {}>", self.limit)
    }
}
pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    _: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    Some(ExclusiveMinimumValidator::compile(schema))
}
