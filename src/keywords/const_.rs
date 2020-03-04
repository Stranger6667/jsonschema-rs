use super::Validate;
use super::{CompilationResult, ValidationResult};
use crate::context::CompilationContext;
use crate::error::ValidationError;
use crate::{helpers, JSONSchema};
use serde_json::{Map, Value};

pub struct ConstValidator<'a> {
    error_message: String,
    value: &'a Value,
}

impl<'a> ConstValidator<'a> {
    pub(crate) fn compile(value: &'a Value) -> CompilationResult<'a> {
        Ok(Box::new(ConstValidator {
            error_message: format!("'{}' was expected", value),
            value,
        }))
    }
}

impl<'a> Validate<'a> for ConstValidator<'a> {
    fn validate(&self, _: &JSONSchema, instance: &Value) -> ValidationResult {
        if !helpers::equal(instance, &self.value) {
            return Err(ValidationError::constant(self.error_message.clone()));
        };
        Ok(())
    }
    fn name(&self) -> String {
        format!("<const: {}>", self.value)
    }
}
pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    _: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    Some(ConstValidator::compile(schema))
}
