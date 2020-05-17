use super::{CompilationResult, Validate};
use crate::{
    compilation::{CompilationContext, JSONSchema},
    error::{error, no_error, CompilationError, ErrorIterator, ValidationError},
};
use serde_json::{Map, Value};
use std::f64::EPSILON;

pub struct MultipleOfFloatValidator {
    multiple_of: f64,
}

impl MultipleOfFloatValidator {
    pub(crate) fn compile(multiple_of: f64) -> CompilationResult {
        Ok(Box::new(MultipleOfFloatValidator { multiple_of }))
    }
}

impl Validate for MultipleOfFloatValidator {
    fn validate<'a>(&self, _: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Some(item) = instance.as_f64() {
            let remainder = (item / self.multiple_of) % 1.;
            if !(remainder < EPSILON && remainder < (1. - EPSILON)) {
                return error(ValidationError::multiple_of(instance, self.multiple_of));
            }
        }
        no_error()
    }

    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Some(item) = instance.as_f64() {
            let remainder = (item / self.multiple_of) % 1.;
            if !(remainder < EPSILON && remainder < (1. - EPSILON)) {
                return false;
            }
        }
        true
    }

    fn name(&self) -> String {
        format!("<multiple of: {}>", self.multiple_of)
    }
}

pub struct MultipleOfIntegerValidator {
    multiple_of: f64,
}

impl MultipleOfIntegerValidator {
    pub(crate) fn compile(multiple_of: f64) -> CompilationResult {
        Ok(Box::new(MultipleOfIntegerValidator { multiple_of }))
    }
}

impl Validate for MultipleOfIntegerValidator {
    fn validate<'a>(&self, _: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Some(item) = instance.as_f64() {
            let is_multiple = if item.fract() == 0. {
                (item % self.multiple_of) == 0.
            } else {
                let remainder = (item / self.multiple_of) % 1.;
                remainder < EPSILON && remainder < (1. - EPSILON)
            };
            if !is_multiple {
                return error(ValidationError::multiple_of(instance, self.multiple_of));
            }
        }
        no_error()
    }

    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Some(item) = instance.as_f64() {
            let is_multiple = if item.fract() == 0. {
                (item % self.multiple_of) == 0.
            } else {
                let remainder = (item / self.multiple_of) % 1.;
                remainder < EPSILON && remainder < (1. - EPSILON)
            };
            if !is_multiple {
                return false;
            }
        }
        true
    }

    fn name(&self) -> String {
        format!("<multiple of: {}>", self.multiple_of)
    }
}

pub(crate) fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    if let Some(multiple_of) = schema.as_f64() {
        return if multiple_of.fract() == 0. {
            Some(MultipleOfIntegerValidator::compile(multiple_of))
        } else {
            Some(MultipleOfFloatValidator::compile(multiple_of))
        };
    }
    Some(Err(CompilationError::SchemaError))
}
