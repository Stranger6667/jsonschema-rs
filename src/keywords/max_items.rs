use super::Validate;
use super::{CompilationResult, ValidationResult};
use crate::context::CompilationContext;
use crate::error::{CompilationError, ValidationError};
use crate::JSONSchema;
use serde_json::{Map, Value};

pub struct MaxItemsValidator {
    limit: usize,
}

impl<'a> MaxItemsValidator {
    pub(crate) fn compile(schema: &Value) -> CompilationResult<'a> {
        if let Value::Number(limit) = schema {
            let limit = limit.as_u64().unwrap() as usize;
            return Ok(Box::new(MaxItemsValidator { limit }));
        }
        Err(CompilationError::SchemaError)
    }
}

impl<'a> Validate<'a> for MaxItemsValidator {
    fn validate(&self, _: &JSONSchema, instance: &Value) -> ValidationResult {
        if let Value::Array(items) = instance {
            if items.len() > self.limit {
                return Err(ValidationError::max_items(instance.clone()));
            }
        }
        Ok(())
    }

    fn name(&self) -> String {
        format!("<max items: {}>", self.limit)
    }
}

pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    _: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    Some(MaxItemsValidator::compile(schema))
}
