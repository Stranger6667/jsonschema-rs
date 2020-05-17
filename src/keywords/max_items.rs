use super::{CompilationResult, Validate};
use crate::{
    compilation::{CompilationContext, JSONSchema},
    error::{error, no_error, CompilationError, ErrorIterator, ValidationError},
};
use serde_json::{Map, Value};

pub struct MaxItemsValidator {
    limit: usize,
}

impl MaxItemsValidator {
    pub(crate) fn compile(schema: &Value) -> CompilationResult {
        if let Some(limit) = schema.as_u64() {
            return Ok(Box::new(MaxItemsValidator {
                limit: limit as usize,
            }));
        }
        Err(CompilationError::SchemaError)
    }
}

impl Validate for MaxItemsValidator {
    fn validate<'a>(&self, _: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Some(items) = instance.as_array() {
            if items.len() > self.limit {
                return error(ValidationError::max_items(instance, self.limit));
            }
        }
        no_error()
    }

    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Some(items) = instance.as_array() {
            if items.len() > self.limit {
                return false;
            }
        }
        true
    }

    fn name(&self) -> String {
        format!("<max items: {}>", self.limit)
    }
}

pub(crate) fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    Some(MaxItemsValidator::compile(schema))
}
