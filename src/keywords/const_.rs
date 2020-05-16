use super::{helpers, CompilationResult, Validate};
use crate::compilation::{CompilationContext, JSONSchema};
use crate::error::{no_error, ErrorIterator, ValidationError};
use serde_json::{Map, Value};

pub struct ConstValidator {
    error_message: String,
    value: Value,
}

impl ConstValidator {
    pub(crate) fn compile(value: &Value) -> CompilationResult {
        Ok(Box::new(ConstValidator {
            error_message: format!("'{}' was expected", value),
            value: value.clone(),
        }))
    }
}

impl Validate for ConstValidator {
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            ValidationError::constant(self.error_message.clone())
        }
    }

    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        helpers::equal(instance, &self.value)
    }

    fn name(&self) -> String {
        format!("<const: {}>", self.value)
    }
}
pub(crate) fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    Some(ConstValidator::compile(schema))
}
