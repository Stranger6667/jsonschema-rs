use super::Validate;
use super::{CompilationResult, ValidationResult};
use crate::context::CompilationContext;
use crate::error::{CompilationError, ValidationError};
use crate::JSONSchema;
use serde_json::{Map, Value};

pub struct ExclusiveMaximumValidator {
    limit: f64,
}

impl<'a> ExclusiveMaximumValidator {
    pub(crate) fn compile(schema: &Value) -> CompilationResult<'a> {
        if let Value::Number(limit) = schema {
            return Ok(Box::new(ExclusiveMaximumValidator {
                limit: limit.as_f64().unwrap(),
            }));
        }
        Err(CompilationError::SchemaError)
    }
}

impl<'a> Validate<'a> for ExclusiveMaximumValidator {
    fn validate(&self, _: &JSONSchema, instance: &Value) -> ValidationResult {
        if let Value::Number(item) = instance {
            let item = item.as_f64().unwrap();
            if item >= self.limit {
                return Err(ValidationError::exclusive_maximum(item, self.limit));
            }
        }
        Ok(())
    }
    fn name(&self) -> String {
        format!("<exclusive maximum: {}>", self.limit)
    }
}

pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    _: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    Some(ExclusiveMaximumValidator::compile(schema))
}
