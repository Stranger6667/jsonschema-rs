use crate::{
    compilation::{CompilationContext, JSONSchema},
    error::{error, no_error, CompilationError, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    validator::Validate,
};
use serde_json::{Map, Value};

pub struct MaxLengthValidator {
    limit: u64,
}

impl MaxLengthValidator {
    #[inline]
    pub(crate) fn compile(schema: &Value) -> CompilationResult {
        if let Some(limit) = schema.as_u64() {
            return Ok(Box::new(MaxLengthValidator { limit }));
        }
        Err(CompilationError::SchemaError)
    }
}

impl Validate for MaxLengthValidator {
    fn validate<'a>(&self, _schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::String(item) = instance {
            if (item.chars().count() as u64) > self.limit {
                return error(ValidationError::max_length(instance, self.limit));
            }
        }
        no_error()
    }

    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            if (item.chars().count() as u64) > self.limit {
                return false;
            }
        }
        true
    }

    fn name(&self) -> String {
        format!("maxLength: {}", self.limit)
    }
}
#[inline]
pub fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    Some(MaxLengthValidator::compile(schema))
}
