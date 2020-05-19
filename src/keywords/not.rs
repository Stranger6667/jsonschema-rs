use super::{CompilationResult, Validate, Validators};
use crate::{
    compilation::{compile_validators, CompilationContext, JSONSchema},
    error::{error, no_error, ErrorIterator, ValidationError},
};
use serde_json::{Map, Value};

pub struct NotValidator {
    // needed only for error representation
    original: Value,
    validators: Validators,
}

impl NotValidator {
    #[inline]
    pub(crate) fn compile(schema: &Value, context: &CompilationContext) -> CompilationResult {
        Ok(Box::new(NotValidator {
            original: schema.clone(),
            validators: compile_validators(schema, context)?,
        }))
    }
}

impl Validate for NotValidator {
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::not(instance, self.original.clone()))
        }
    }

    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        !self
            .validators
            .iter()
            .all(|validator| validator.is_valid(schema, instance))
    }

    fn name(&self) -> String {
        format!("<not: {:?}>", self.validators)
    }
}

#[inline]
pub(crate) fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    Some(NotValidator::compile(schema, context))
}
