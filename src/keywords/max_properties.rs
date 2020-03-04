use super::Validate;
use super::{CompilationResult, ValidationResult};
use crate::context::CompilationContext;
use crate::error::{CompilationError, ValidationError};
use crate::JSONSchema;
use serde_json::{Map, Value};

pub struct MaxPropertiesValidator {
    limit: usize,
}

impl<'a> MaxPropertiesValidator {
    pub(crate) fn compile(schema: &Value) -> CompilationResult<'a> {
        if let Value::Number(limit) = schema {
            let limit = limit.as_u64().unwrap() as usize;
            return Ok(Box::new(MaxPropertiesValidator { limit }));
        }
        Err(CompilationError::SchemaError)
    }
}

impl<'a> Validate<'a> for MaxPropertiesValidator {
    fn validate(&self, _: &JSONSchema, instance: &Value) -> ValidationResult {
        if let Value::Object(item) = instance {
            if item.len() > self.limit {
                return Err(ValidationError::max_properties(instance.clone()));
            }
        }
        Ok(())
    }
    fn name(&self) -> String {
        format!("<max properties: {}>", self.limit)
    }
}
pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    _: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    Some(MaxPropertiesValidator::compile(schema))
}
