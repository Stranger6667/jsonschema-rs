use super::{helpers, CompilationResult, Validate};
use crate::{
    compilation::{CompilationContext, JSONSchema},
    error::{error, no_error, ErrorIterator, ValidationError},
};
use serde_json::{Map, Value};

pub struct ConstValidator {
    value: Value,
}

impl ConstValidator {
    #[inline]
    pub(crate) fn compile(value: &Value) -> CompilationResult {
        Ok(Box::new(ConstValidator {
            value: value.clone(),
        }))
    }
}

impl Validate for ConstValidator {
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::constant(instance, &self.value))
        }
    }

    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        helpers::equal(instance, &self.value)
    }

    fn name(&self) -> String {
        format!("<const: {}>", self.value)
    }
}
#[inline]
pub(crate) fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    Some(ConstValidator::compile(schema))
}
