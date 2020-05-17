use super::{CompilationResult, Validate, Validators};
use crate::{
    compilation::{compile_validators, CompilationContext, JSONSchema},
    error::{error, no_error, ErrorIterator, ValidationError},
};
use serde_json::{Map, Value};

pub struct ContainsValidator {
    validators: Validators,
}

impl ContainsValidator {
    pub(crate) fn compile(schema: &Value, context: &CompilationContext) -> CompilationResult {
        Ok(Box::new(ContainsValidator {
            validators: compile_validators(schema, context)?,
        }))
    }
}

impl Validate for ContainsValidator {
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Some(items) = instance.as_array() {
            for item in items {
                if self
                    .validators
                    .iter()
                    .all(|validator| validator.is_valid(schema, item))
                {
                    return no_error();
                }
            }
            return error(ValidationError::contains(instance));
        }
        no_error()
    }

    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Some(items) = instance.as_array() {
            for item in items {
                if self
                    .validators
                    .iter()
                    .all(|validator| validator.is_valid(schema, item))
                {
                    return true;
                }
            }
            return false;
        }
        true
    }

    fn name(&self) -> String {
        format!("<contains: {:?}>", self.validators)
    }
}

pub(crate) fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    Some(ContainsValidator::compile(schema, context))
}
