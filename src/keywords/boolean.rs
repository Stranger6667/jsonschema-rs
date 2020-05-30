use crate::{
    compilation::JSONSchema,
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    validator::Validate,
};
use serde_json::{Map, Value};

pub struct TrueValidator {}
impl TrueValidator {
    #[inline]
    pub(crate) fn compile() -> CompilationResult {
        Ok(Box::new(TrueValidator {}))
    }
}
impl Validate for TrueValidator {
    fn name(&self) -> String {
        "true".to_string()
    }

    #[inline]
    fn is_valid(&self, _: &JSONSchema, _: &Value) -> bool {
        true
    }

    #[inline]
    fn validate<'a>(&self, _: &'a JSONSchema, _: &'a Value) -> ErrorIterator<'a> {
        no_error()
    }
}

pub struct FalseValidator {}

impl FalseValidator {
    #[inline]
    pub(crate) fn compile() -> CompilationResult {
        Ok(Box::new(FalseValidator {}))
    }
}

impl Validate for FalseValidator {
    #[inline]
    fn build_validation_error<'a>(&self, instance: &'a Value) -> ValidationError<'a> {
        ValidationError::false_schema(instance)
    }

    fn name(&self) -> String {
        "false".to_string()
    }

    #[inline]
    fn is_valid_array(&self, _: &JSONSchema, _: &Value, _: &[Value]) -> bool {
        false
    }
    #[inline]
    fn is_valid_boolean(&self, _: &JSONSchema, _: &Value, _: bool) -> bool {
        false
    }
    #[inline]
    fn is_valid_object(&self, _: &JSONSchema, _: &Value, _: &Map<String, Value>) -> bool {
        false
    }
    #[inline]
    fn is_valid_null(&self, _: &JSONSchema, _: &Value, _: ()) -> bool {
        false
    }
    #[inline]
    fn is_valid_number(&self, _: &JSONSchema, _: &Value, _: f64) -> bool {
        false
    }
    #[inline]
    fn is_valid_signed_integer(&self, _: &JSONSchema, _: &Value, _: i64) -> bool {
        false
    }
    #[inline]
    fn is_valid_string(&self, _: &JSONSchema, _: &Value, _: &str) -> bool {
        false
    }
    #[inline]
    fn is_valid_unsigned_integer(&self, _: &JSONSchema, _: &Value, _: u64) -> bool {
        false
    }
    #[inline]
    fn is_valid(&self, _: &JSONSchema, _: &Value) -> bool {
        false
    }

    #[inline]
    fn validate<'a>(&self, _: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        error(self.build_validation_error(instance))
    }
}

#[inline]
pub fn compile(value: bool) -> Option<CompilationResult> {
    if value {
        Some(TrueValidator::compile())
    } else {
        Some(FalseValidator::compile())
    }
}
