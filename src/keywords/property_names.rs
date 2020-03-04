use super::{CompilationResult, ValidationResult};
use super::{Validate, Validators};
use crate::context::CompilationContext;
use crate::error::ValidationError;
use crate::validator::compile_validators;
use crate::JSONSchema;
use serde_json::{Map, Value};

pub struct PropertyNamesObjectValidator<'a> {
    validators: Validators<'a>,
}

impl<'a> PropertyNamesObjectValidator<'a> {
    pub(crate) fn compile(
        schema: &'a Value,
        context: &CompilationContext,
    ) -> CompilationResult<'a> {
        Ok(Box::new(PropertyNamesObjectValidator {
            validators: compile_validators(schema, context)?,
        }))
    }
}

impl<'a> Validate<'a> for PropertyNamesObjectValidator<'a> {
    fn validate(&self, config: &JSONSchema, instance: &Value) -> ValidationResult {
        if let Value::Object(item) = instance {
            for name in item.keys() {
                let wrapper = Value::String(name.to_string());
                for validator in self.validators.iter() {
                    validator.validate(config, &wrapper)?
                }
            }
        }
        Ok(())
    }

    fn name(&self) -> String {
        format!("<property names: {:?}>", self.validators)
    }
}

pub struct PropertyNamesBooleanValidator {}

impl<'a> PropertyNamesBooleanValidator {
    pub(crate) fn compile() -> CompilationResult<'a> {
        Ok(Box::new(PropertyNamesBooleanValidator {}))
    }
}

impl<'a> Validate<'a> for PropertyNamesBooleanValidator {
    fn validate(&self, _: &JSONSchema, instance: &Value) -> ValidationResult {
        if let Value::Object(item) = instance {
            if !item.is_empty() {
                return Err(ValidationError::false_schema(instance.clone()));
            }
        }
        Ok(())
    }

    fn name(&self) -> String {
        "<property names: false>".to_string()
    }
}

pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    match schema {
        Value::Object(_) => Some(PropertyNamesObjectValidator::compile(schema, context)),
        Value::Bool(false) => Some(PropertyNamesBooleanValidator::compile()),
        _ => None,
    }
}
