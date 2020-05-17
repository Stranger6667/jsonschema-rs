use super::{CompilationResult, Validate};
use crate::{
    compilation::{CompilationContext, JSONSchema},
    error::{error, no_error, CompilationError, ErrorIterator, ValidationError},
};
use serde_json::{Map, Value};

pub struct MinPropertiesValidator {
    limit: usize,
}

impl<'a> MinPropertiesValidator {
    pub(crate) fn compile(schema: &Value) -> CompilationResult {
        if let Some(limit) = schema.as_u64() {
            return Ok(Box::new(MinPropertiesValidator {
                limit: limit as usize,
            }));
        }
        Err(CompilationError::SchemaError)
    }
}

impl Validate for MinPropertiesValidator {
    fn validate<'a>(&self, _: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Some(item) = instance.as_object() {
            if item.len() < self.limit {
                return error(ValidationError::min_properties(instance, self.limit));
            }
        }
        no_error()
    }

    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Some(item) = instance.as_object() {
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
