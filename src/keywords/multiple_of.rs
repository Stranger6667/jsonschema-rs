use super::Validate;
use super::{CompilationResult, ValidationResult};
use crate::context::CompilationContext;
use crate::error::{CompilationError, ValidationError};
use crate::JSONSchema;
use serde_json::{Map, Value};
use std::f64::EPSILON;

pub struct MultipleOfFloatValidator {
    multiple_of: f64,
}

impl<'a> MultipleOfFloatValidator {
    pub(crate) fn compile(multiple_of: f64) -> CompilationResult<'a> {
        Ok(Box::new(MultipleOfFloatValidator { multiple_of }))
    }
}

impl<'a> Validate<'a> for MultipleOfFloatValidator {
    fn validate(&self, _: &JSONSchema, instance: &Value) -> ValidationResult {
        if let Value::Number(item) = instance {
            let item = item.as_f64().unwrap();
            let remainder = (item / self.multiple_of) % 1.;
            if !(remainder < EPSILON && remainder < (1. - EPSILON)) {
                return Err(ValidationError::multiple_of(item, self.multiple_of));
            }
        }
        Ok(())
    }
    fn name(&self) -> String {
        format!("<multiple of: {}>", self.multiple_of)
    }
}

pub struct MultipleOfIntegerValidator {
    multiple_of: f64,
}

impl<'a> MultipleOfIntegerValidator {
    pub(crate) fn compile(multiple_of: f64) -> CompilationResult<'a> {
        Ok(Box::new(MultipleOfIntegerValidator { multiple_of }))
    }
}

impl<'a> Validate<'a> for MultipleOfIntegerValidator {
    fn validate(&self, _: &JSONSchema, instance: &Value) -> ValidationResult {
        if let Value::Number(item) = instance {
            let item = item.as_f64().unwrap();
            let is_multiple = if item.fract() == 0. {
                (item % self.multiple_of) == 0.
            } else {
                let remainder = (item / self.multiple_of) % 1.;
                remainder < EPSILON && remainder < (1. - EPSILON)
            };
            if !is_multiple {
                return Err(ValidationError::multiple_of(item, self.multiple_of));
            }
        }
        Ok(())
    }
    fn name(&self) -> String {
        format!("<multiple of: {}>", self.multiple_of)
    }
}

pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    _: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    if let Value::Number(multiple_of) = schema {
        let multiple_of = multiple_of.as_f64().unwrap();
        return if multiple_of.fract() == 0. {
            Some(MultipleOfIntegerValidator::compile(multiple_of))
        } else {
            Some(MultipleOfFloatValidator::compile(multiple_of))
        };
    }
    Some(Err(CompilationError::SchemaError))
}
