use crate::{
    compilation::{context::CompilationContext, JSONSchema},
    error::{no_error, CompilationError, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    validator::Validate,
};
use serde_json::{Map, Value};

pub struct MaxItemsValidator {
    limit: u64,
}

impl MaxItemsValidator {
    #[inline]
    pub(crate) fn compile(schema: &Value) -> CompilationResult {
        if let Some(limit) = schema.as_u64() {
            return Ok(Box::new(MaxItemsValidator { limit }));
        }
        Err(CompilationError::SchemaError)
    }
}

impl Validate for MaxItemsValidator {
    #[inline]
    fn build_validation_error<'a>(&self, instance: &'a Value) -> ValidationError<'a> {
        ValidationError::max_items(instance, self.limit)
    }

    #[inline]
    fn is_valid_array(&self, _: &JSONSchema, _: &Value, instance_value: &[Value]) -> bool {
        instance_value.len() as u64 <= self.limit
    }
    #[inline]
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::Array(instance_value) = instance {
            instance_value.len() as u64 <= self.limit
        } else {
            true
        }
    }

    #[inline]
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Array(instance_value) = instance {
            self.validate_array(schema, instance, instance_value)
        } else {
            no_error()
        }
    }
}
impl ToString for MaxItemsValidator {
    fn to_string(&self) -> String {
        format!("maxItems: {}", self.limit)
    }
}

#[inline]
pub fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    Some(MaxItemsValidator::compile(schema))
}
