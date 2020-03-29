use super::CompilationResult;
use super::Validate;
use crate::compilation::CompilationContext;
use crate::error::{no_error, CompilationError, ErrorIterator, ValidationError};
use crate::JSONSchema;
use serde_json::{Map, Value};

pub struct MinPropertiesValidator {
    limit: usize,
}

impl<'a> MinPropertiesValidator {
    pub(crate) fn compile(schema: &Value) -> CompilationResult {
        if let Value::Number(limit) = schema {
            let limit = limit.as_u64().unwrap() as usize;
            return Ok(Box::new(MinPropertiesValidator { limit }));
        }
        Err(CompilationError::SchemaError)
    }
}

impl Validate for MinPropertiesValidator {
    fn validate<'a>(&self, _: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Object(item) = instance {
            if item.len() < self.limit {
                return ValidationError::min_properties(instance.clone());
            }
        }
        no_error()
    }

    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            if item.len() < self.limit {
                return false;
            }
        }
        true
    }

    fn name(&self) -> String {
        format!("<min properties: {}>", self.limit)
    }
}

pub(crate) fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    Some(MinPropertiesValidator::compile(schema))
}
