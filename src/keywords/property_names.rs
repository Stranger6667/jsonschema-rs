use super::{CompilationResult, Validate, Validators};
use crate::{
    compilation::{compile_validators, CompilationContext, JSONSchema},
    error::{error, no_error, ErrorIterator, ValidationError},
};
use serde_json::{Map, Value};

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
        if let Some(item) = instance.as_object() {
            let errors: Vec<_> = self
                .validators
                .iter()
                .flat_map(move |validator| {
                    item.keys().flat_map(move |key| {
                        let wrapper = Value::String(key.to_string());
                        let errors: Vec<_> = validator
                            .validate(schema, &wrapper)
                            .map(|validation_error| validation_error.into_owned())
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
        if let Some(item) = instance.as_object() {
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
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::false_schema(instance))
        }
    }

    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Some(item) = instance.as_object() {
            if !item.is_empty() {
                return false;
            }
        }
        true
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
    if schema.is_object() {
        Some(PropertyNamesObjectValidator::compile(schema, context))
    } else if let Some(value) = schema.as_bool() {
        if value {
            None
        } else {
            Some(PropertyNamesBooleanValidator::compile())
        }
    } else {
        None
    }
}
