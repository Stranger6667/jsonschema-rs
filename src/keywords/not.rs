use super::CompilationResult;
use super::{Validate, Validators};
use crate::context::CompilationContext;
use crate::error::{no_error, ErrorIterator, ValidationError};
use crate::validator::compile_validators;
use crate::JSONSchema;
use serde_json::{Map, Value};

pub struct NotValidator {
    // needed only for error representation
    original: Value,
    validators: Validators,
}

impl NotValidator {
    pub(crate) fn compile(schema: &Value, context: &CompilationContext) -> CompilationResult {
        Ok(Box::new(NotValidator {
            original: schema.clone(),
            validators: compile_validators(schema, context)?,
        }))
    }
}

impl Validate for NotValidator {
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if self
            .validators
            .iter()
            .all(|validator| validator.is_valid(schema, instance))
        {
            ValidationError::not(instance.clone(), self.original.clone())
        } else {
            no_error()
        }
    }
    fn name(&self) -> String {
        format!("<not: {:?}>", self.validators)
    }
}

pub(crate) fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    Some(NotValidator::compile(schema, context))
}
