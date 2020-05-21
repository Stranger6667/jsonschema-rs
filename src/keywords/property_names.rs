use super::{CompilationResult, Validate, Validators};
use crate::{
    compilation::{compile_validators, CompilationContext, JSONSchema},
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::format_validators,
};
use serde_json::{Map, Value};
use std::borrow::Borrow;

pub struct PropertyNamesObjectValidator {
    validators: Validators,
}

impl PropertyNamesObjectValidator {
    #[inline]
    pub(crate) fn compile(schema: &Value, context: &CompilationContext) -> CompilationResult {
        Ok(Box::new(PropertyNamesObjectValidator {
            validators: compile_validators(schema, context)?,
        }))
    }
}

impl Validate for PropertyNamesObjectValidator {
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Object(item) = &instance.borrow() {
            let errors: Vec<_> = self
                .validators
                .iter()
                .flat_map(move |validator| {
                    item.keys().flat_map(move |key| {
                        let wrapper = Value::String(key.to_string());
                        let errors: Vec<_> = validator
                            .validate(schema, &wrapper)
                            .map(ValidationError::into_owned)
                            .collect();
                        errors.into_iter()
                    })
                })
                .collect();
            return Box::new(errors.into_iter());
        }
        no_error()
    }

    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(item) = &instance.borrow() {
            return self.validators.iter().all(move |validator| {
                item.keys().all(move |key| {
                    let wrapper = Value::String(key.to_string());
                    validator.is_valid(schema, &wrapper)
                })
            });
        }
        true
    }

    fn name(&self) -> String {
        format!("propertyNames: {}", format_validators(&self.validators))
    }
}

pub struct PropertyNamesBooleanValidator {}

impl PropertyNamesBooleanValidator {
    #[inline]
    pub(crate) fn compile() -> CompilationResult {
        Ok(Box::new(PropertyNamesBooleanValidator {}))
    }
}

impl Validate for PropertyNamesBooleanValidator {
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::false_schema(instance))
        }
    }

    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            if !item.is_empty() {
                return false;
            }
        }
        true
    }

    fn name(&self) -> String {
        "propertyNames: false".to_string()
    }
}

#[inline]
pub fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    match schema {
        Value::Object(_) => Some(PropertyNamesObjectValidator::compile(schema, context)),
        Value::Bool(false) => Some(PropertyNamesBooleanValidator::compile()),
        _ => None,
    }
}
