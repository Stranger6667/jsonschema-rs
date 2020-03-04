use super::Validate;
use super::{CompilationResult, ValidationResult};
use crate::context::CompilationContext;
use crate::error::{CompilationError, ValidationError};
use crate::JSONSchema;
use serde_json::{Map, Value};

pub struct MaxLengthValidator {
    limit: usize,
}

impl<'a> MaxLengthValidator {
    pub(crate) fn compile(schema: &Value) -> CompilationResult<'a> {
        if let Value::Number(limit) = schema {
            let limit = limit.as_u64().unwrap() as usize;
            return Ok(Box::new(MaxLengthValidator { limit }));
        }
        Err(CompilationError::SchemaError)
    }
}

impl<'a> Validate<'a> for MaxLengthValidator {
    fn validate(&self, _: &JSONSchema, instance: &Value) -> ValidationResult {
        if let Value::String(item) = instance {
            if item.chars().count() > self.limit {
                return Err(ValidationError::max_length(item.clone()));
            }
        }
        Ok(())
    }
    fn name(&self) -> String {
        format!("<max length: {}>", self.limit)
    }
}
pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    _: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    Some(MaxLengthValidator::compile(schema))
}
