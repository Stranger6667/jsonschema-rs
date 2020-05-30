use crate::{
    compilation::{CompilationContext, JSONSchema},
    error::ValidationError,
    keywords::CompilationResult,
    validator::Validate,
};
use serde_json::{Map, Value};
use std::f64::EPSILON;

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
    fn build_validation_error<'a>(&self, instance: &'a Value) -> ValidationError<'a> {
        ValidationError::constant(instance, &self.value)
    }

    fn name(&self) -> String {
        format!("const: {}", self.value)
    }

    #[inline]
    fn is_valid_array(&self, _: &JSONSchema, _: &Value, instance_value: &[Value]) -> bool {
        self.value
            .as_array()
            .map_or_else(|| false, |value| value.as_slice() == instance_value)
    }
    #[inline]
    fn is_valid_boolean(&self, _: &JSONSchema, _: &Value, instance_value: bool) -> bool {
        self.value
            .as_bool()
            .map_or_else(|| false, |value| value == instance_value)
    }
    #[inline]
    fn is_valid_object(
        &self,
        _: &JSONSchema,
        _: &Value,
        instance_value: &Map<String, Value>,
    ) -> bool {
        self.value
            .as_object()
            .map_or_else(|| false, |value| value == instance_value)
    }
    #[inline]
    fn is_valid_null(&self, _: &JSONSchema, _: &Value, _: ()) -> bool {
        self.value.is_null()
    }
    #[inline]
    fn is_valid_number(&self, _: &JSONSchema, _: &Value, instance_value: f64) -> bool {
        self.value
            .as_f64()
            .map_or_else(|| false, |value| (value - instance_value).abs() < EPSILON)
    }
    #[inline]
    fn is_valid_signed_integer(
        &self,
        schema: &JSONSchema,
        instance: &Value,
        instance_value: i64,
    ) -> bool {
        #[allow(clippy::cast_precision_loss)]
        self.is_valid_number(schema, instance, instance_value as f64)
    }
    #[inline]
    fn is_valid_string(&self, _: &JSONSchema, _: &Value, instance_value: &str) -> bool {
        self.value
            .as_str()
            .map_or_else(|| false, |value| value == instance_value)
    }
    #[inline]
    fn is_valid_unsigned_integer(
        &self,
        schema: &JSONSchema,
        instance: &Value,
        instance_value: u64,
    ) -> bool {
        #[allow(clippy::cast_precision_loss)]
        self.is_valid_number(schema, instance, instance_value as f64)
    }
}

#[inline]
pub fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    Some(ConstValidator::compile(schema))
}
