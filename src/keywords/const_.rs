use super::CompilationResult;
use super::Validate;
use crate::context::CompilationContext;
use crate::error::{no_error, ErrorIterator, ValidationError};
use crate::{helpers, JSONSchema};
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
    fn validate<'a>(&self, _: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if !helpers::equal(instance, &self.value) {
            return ValidationError::constant(self.error_message.clone());
        };
        no_error()
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
