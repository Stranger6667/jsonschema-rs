use crate::{
    compilation::{context::CompilationContext, JSONSchema},
    error::{no_error, CompilationError, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    validator::Validate,
};
use serde_json::{Map, Value};
use std::f64::EPSILON;

pub(crate) struct MultipleOfFloatValidator {
    multiple_of: f64,
}

impl MultipleOfFloatValidator {
    #[inline]
    pub(crate) fn compile(multiple_of: f64) -> CompilationResult {
        Ok(Box::new(MultipleOfFloatValidator { multiple_of }))
    }
}

impl Validate for MultipleOfFloatValidator {
    #[inline]
    fn build_validation_error<'a>(&self, instance: &'a Value) -> ValidationError<'a> {
        ValidationError::multiple_of(instance, self.multiple_of)
    }

    #[inline]
    fn is_valid_number(&self, _: &JSONSchema, _: &Value, instance_value: f64) -> bool {
        let remainder = (instance_value / self.multiple_of) % 1.;
        remainder < EPSILON && remainder < (1. - EPSILON)
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
    fn is_valid_unsigned_integer(
        &self,
        schema: &JSONSchema,
        instance: &Value,
        instance_value: u64,
    ) -> bool {
        #[allow(clippy::cast_precision_loss)]
        self.is_valid_number(schema, instance, instance_value as f64)
    }
    #[inline]
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Some(instance_value) = instance.as_f64() {
            self.is_valid_number(schema, instance, instance_value)
        } else {
            true
        }
    }

    #[inline]
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Some(instance_value) = instance.as_f64() {
            self.validate_number(schema, instance, instance_value)
        } else {
            no_error()
        }
    }
}
impl ToString for MultipleOfFloatValidator {
    fn to_string(&self) -> String {
        format!("multipleOf: {}", self.multiple_of)
    }
}

pub(crate) struct MultipleOfIntegerValidator {
    multiple_of: f64,
}

impl MultipleOfIntegerValidator {
    #[inline]
    pub(crate) fn compile(multiple_of: f64) -> CompilationResult {
        Ok(Box::new(MultipleOfIntegerValidator { multiple_of }))
    }
}

impl Validate for MultipleOfIntegerValidator {
    #[inline]
    fn build_validation_error<'a>(&self, instance: &'a Value) -> ValidationError<'a> {
        ValidationError::multiple_of(instance, self.multiple_of)
    }

    #[inline]
    fn is_valid_number(&self, _: &JSONSchema, _: &Value, instance_value: f64) -> bool {
        if instance_value.fract() == 0. {
            (instance_value % self.multiple_of) == 0.
        } else {
            let remainder = (instance_value / self.multiple_of) % 1.;
            remainder < EPSILON && remainder < (1. - EPSILON)
        }
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
    fn is_valid_unsigned_integer(
        &self,
        schema: &JSONSchema,
        instance: &Value,
        instance_value: u64,
    ) -> bool {
        #[allow(clippy::cast_precision_loss)]
        self.is_valid_number(schema, instance, instance_value as f64)
    }
    #[inline]
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Some(instance_value) = instance.as_f64() {
            self.is_valid_number(schema, instance, instance_value)
        } else {
            true
        }
    }

    #[inline]
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Some(instance_value) = instance.as_f64() {
            self.validate_number(schema, instance, instance_value)
        } else {
            no_error()
        }
    }
}
impl ToString for MultipleOfIntegerValidator {
    fn to_string(&self) -> String {
        format!("multipleOf: {}", self.multiple_of)
    }
}

#[inline]
pub(crate) fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    if let Value::Number(multiple_of) = schema {
        let multiple_of = multiple_of.as_f64().expect("Always valid");
        return if multiple_of.fract() == 0. {
            Some(MultipleOfIntegerValidator::compile(multiple_of))
        } else {
            Some(MultipleOfFloatValidator::compile(multiple_of))
        };
    }
    Some(Err(CompilationError::SchemaError))
}
