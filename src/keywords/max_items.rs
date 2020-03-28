use super::CompilationResult;
use super::Validate;
use crate::context::CompilationContext;
use crate::error::{no_error, CompilationError, ErrorIterator, ValidationError};
use crate::JSONSchema;
use serde_json::{Map, Value};

pub struct MaxItemsValidator {
    limit: usize,
}

impl<'a> MaxItemsValidator {
    pub(crate) fn compile(schema: &Value) -> CompilationResult {
        if let Value::Number(limit) = schema {
            let limit = limit.as_u64().unwrap() as usize;
            return Ok(Box::new(MaxItemsValidator { limit }));
        }
        Err(CompilationError::SchemaError)
    }
}

impl Validate for MaxItemsValidator {
    fn validate<'a>(&self, _: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Array(items) = instance {
            if items.len() > self.limit {
                return ValidationError::max_items(instance.clone());
            }
        }
        no_error()
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
