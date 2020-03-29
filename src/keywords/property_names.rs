use super::{CompilationResult, Validate, Validators};
use crate::compilation::{compile_validators, CompilationContext, JSONSchema};
use crate::error::{no_error, ErrorIterator, ValidationError};
use serde_json::{Map, Value};
use std::borrow::Borrow;

pub struct PropertyNamesObjectValidator {
    validators: Validators,
}

impl PropertyNamesObjectValidator {
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
                        let errors: Vec<_> = validator.validate(schema, &wrapper).collect();
                        errors.into_iter()
                    })
                })
                .collect();
            return Box::new(errors.into_iter());
        }
        no_error()
    }

    fn name(&self) -> String {
        format!("<property names: {:?}>", self.validators)
    }
}

pub struct PropertyNamesBooleanValidator {}

impl PropertyNamesBooleanValidator {
    pub(crate) fn compile() -> CompilationResult {
        Ok(Box::new(PropertyNamesBooleanValidator {}))
    }
}

impl Validate for PropertyNamesBooleanValidator {
    fn validate<'a>(&self, _: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Object(item) = instance.borrow() {
            if !item.is_empty() {
                return ValidationError::false_schema(instance.clone());
            }
        }
        no_error()
    }

    fn name(&self) -> String {
        "<property names: false>".to_string()
    }
}

pub(crate) fn compile(
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
