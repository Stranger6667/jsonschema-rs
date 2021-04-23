use crate::keywords::InstancePath;

use crate::{
    compilation::JSONSchema,
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    validator::Validate,
};
use serde_json::Value;

pub(crate) struct TrueValidator {}
impl TrueValidator {
    #[inline]
    pub(crate) fn compile() -> CompilationResult {
        Ok(Box::new(TrueValidator {}))
    }
}
impl Validate for TrueValidator {
    fn is_valid(&self, _: &JSONSchema, _: &Value) -> bool {
        true
    }

    fn validate<'a, 'b>(
        &'b self,
        _: &'a JSONSchema,
        _: &'a Value,
        _: &InstancePath<'b>,
    ) -> ErrorIterator<'a> {
        no_error()
    }
}

impl ToString for TrueValidator {
    fn to_string(&self) -> String {
        "true".to_string()
    }
}

pub(crate) struct FalseValidator {}
impl FalseValidator {
    #[inline]
    pub(crate) fn compile() -> CompilationResult {
        Ok(Box::new(FalseValidator {}))
    }
}
impl Validate for FalseValidator {
    fn is_valid(&self, _: &JSONSchema, _: &Value) -> bool {
        false
    }

    fn validate<'a, 'b>(
        &'b self,
        _: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath<'b>,
    ) -> ErrorIterator<'a> {
        error(ValidationError::false_schema(
            instance_path.into(),
            instance,
        ))
    }
}

impl ToString for FalseValidator {
    fn to_string(&self) -> String {
        "false".to_string()
    }
}

#[inline]
pub(crate) fn compile(value: bool) -> Option<CompilationResult> {
    if value {
        Some(TrueValidator::compile())
    } else {
        Some(FalseValidator::compile())
    }
}
