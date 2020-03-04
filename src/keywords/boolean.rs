use super::Validate;
use super::{CompilationResult, ValidationResult};
use crate::error::ValidationError;
use crate::JSONSchema;
use serde_json::Value;

pub struct TrueValidator {}

impl TrueValidator {
    pub(crate) fn compile<'a>() -> CompilationResult<'a> {
        Ok(Box::new(TrueValidator {}))
    }
}

impl<'a> Validate<'a> for TrueValidator {
    fn validate(&self, _: &JSONSchema, _: &Value) -> ValidationResult {
        Ok(())
    }
    fn name(&self) -> String {
        "<true>".to_string()
    }
}

pub struct FalseValidator {}

impl FalseValidator {
    pub(crate) fn compile<'a>() -> CompilationResult<'a> {
        Ok(Box::new(FalseValidator {}))
    }
}

impl<'a> Validate<'a> for FalseValidator {
    fn validate(&self, _: &JSONSchema, instance: &Value) -> ValidationResult {
        Err(ValidationError::false_schema(instance.clone()))
    }
    fn name(&self) -> String {
        "<false>".to_string()
    }
}

pub(crate) fn compile<'a>(value: bool) -> Option<CompilationResult<'a>> {
    if value {
        Some(TrueValidator::compile())
    } else {
        Some(FalseValidator::compile())
    }
}
