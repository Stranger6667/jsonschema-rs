use super::Validate;
use super::{CompilationResult, ValidationResult};
use crate::context::CompilationContext;
use crate::error::{CompilationError, ValidationError};
use crate::JSONSchema;
use serde_json::{Map, Value};

pub struct MinLengthValidator {
    limit: usize,
}

impl<'a> MinLengthValidator {
    pub(crate) fn compile(schema: &Value) -> CompilationResult<'a> {
        if let Value::Number(limit) = schema {
            let limit = limit.as_u64().unwrap() as usize;
            return Ok(Box::new(MinLengthValidator { limit }));
        }
        Err(CompilationError::SchemaError)
    }
}

impl<'a> Validate<'a> for MinLengthValidator {
    fn validate(&self, _: &JSONSchema, instance: &Value) -> ValidationResult {
        if let Value::String(item) = instance {
            if item.chars().count() < self.limit {
                return Err(ValidationError::min_length(item.clone()));
            }
        }
        Ok(())
    }

    fn name(&self) -> String {
        format!("<min length: {}>", self.limit)
    }
}
pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    _: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    Some(MinLengthValidator::compile(schema))
}
