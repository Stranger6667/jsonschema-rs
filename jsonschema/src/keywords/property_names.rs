use crate::{
    compilation::{compile_validators, context::CompilationContext, JSONSchema},
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::{format_validators, CompilationResult, InstancePath, Validators},
    validator::Validate,
};
use serde_json::{Map, Value};
use std::borrow::Borrow;

pub(crate) struct PropertyNamesObjectValidator {
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
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(item) = &instance.borrow() {
            self.validators.iter().all(move |validator| {
                item.keys().all(move |key| {
                    let wrapper = Value::String(key.to_string());
                    validator.is_valid(schema, &wrapper)
                })
            })
        } else {
            true
        }
    }

    fn validate<'a, 'b>(
        &'b self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: InstancePath<'b>,
    ) -> ErrorIterator<'a> {
        if let Value::Object(item) = &instance.borrow() {
            let errors: Vec<_> = self
                .validators
                .iter()
                .flat_map(move |validator| {
                    let instance_path = instance_path.clone();
                    item.keys().flat_map(move |key| {
                        let wrapper = Value::String(key.to_string());
                        let errors: Vec<_> = validator
                            .validate(schema, &wrapper, instance_path.clone())
                            .map(ValidationError::into_owned)
                            .collect();
                        errors.into_iter()
                    })
                })
                .collect();
            Box::new(errors.into_iter())
        } else {
            no_error()
        }
    }
}
impl ToString for PropertyNamesObjectValidator {
    fn to_string(&self) -> String {
        format!("propertyNames: {}", format_validators(&self.validators))
    }
}

pub(crate) struct PropertyNamesBooleanValidator {}

impl PropertyNamesBooleanValidator {
    #[inline]
    pub(crate) fn compile() -> CompilationResult {
        Ok(Box::new(PropertyNamesBooleanValidator {}))
    }
}

impl Validate for PropertyNamesBooleanValidator {
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            if !item.is_empty() {
                return false;
            }
        }
        true
    }

    fn validate<'a, 'b>(
        &'b self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        _: InstancePath<'b>,
    ) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::false_schema(instance))
        }
    }
}

impl ToString for PropertyNamesBooleanValidator {
    fn to_string(&self) -> String {
        "propertyNames: false".to_string()
    }
}

#[inline]
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
