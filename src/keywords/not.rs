use super::{CompilationResult, ValidationResult};
use super::{Validate, Validators};
use crate::context::CompilationContext;
use crate::error::ValidationError;
use crate::validator::compile_validators;
use crate::JSONSchema;
use serde_json::{Map, Value};

pub struct NotValidator<'a> {
    // needed only for error representation
    original: &'a Value,
    validators: Validators<'a>,
}

impl<'a> NotValidator<'a> {
    pub(crate) fn compile(
        schema: &'a Value,
        context: &CompilationContext,
    ) -> CompilationResult<'a> {
        Ok(Box::new(NotValidator {
            original: schema,
            validators: compile_validators(schema, context)?,
        }))
    }
}

impl<'a> Validate<'a> for NotValidator<'a> {
    fn validate(&self, config: &JSONSchema, instance: &Value) -> ValidationResult {
        if self
            .validators
            .iter()
            .all(|validator| validator.is_valid(config, instance))
        {
            Err(ValidationError::not(
                instance.clone(),
                self.original.clone(),
            ))
        } else {
            Ok(())
        }
    }
    fn name(&self) -> String {
        format!("<not: {:?}>", self.validators)
    }
}

pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    Some(NotValidator::compile(schema, context))
}
