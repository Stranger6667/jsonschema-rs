use super::{CompilationResult, Validate};
use crate::compilation::JSONSchema;
use crate::error::{no_error, ErrorIterator, ValidationError};
use serde_json::Value;

pub struct TrueValidator {}

impl TrueValidator {
    pub(crate) fn compile() -> CompilationResult {
        Ok(Box::new(TrueValidator {}))
    }
}

impl Validate for TrueValidator {
    fn validate<'a>(&self, _: &'a JSONSchema, _: &'a Value) -> ErrorIterator<'a> {
        no_error()
    }

    fn is_valid(&self, _: &JSONSchema, _: &Value) -> bool {
        true
    }

    fn name(&self) -> String {
        "<true>".to_string()
    }
}

pub struct FalseValidator {}

impl FalseValidator {
    pub(crate) fn compile() -> CompilationResult {
        Ok(Box::new(FalseValidator {}))
    }
}

impl Validate for FalseValidator {
    fn validate<'a>(&self, _: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        ValidationError::false_schema(instance.clone())
    }

    fn is_valid(&self, _: &JSONSchema, _: &Value) -> bool {
        false
    }

    fn name(&self) -> String {
        "<false>".to_string()
    }
}

pub(crate) fn compile(value: bool) -> Option<CompilationResult> {
    if value {
        Some(TrueValidator::compile())
    } else {
        Some(FalseValidator::compile())
    }
}
