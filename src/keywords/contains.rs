use super::{CompilationResult, ValidationResult};
use super::{Validate, Validators};
use crate::context::CompilationContext;
use crate::error::ValidationError;
use crate::validator::compile_validators;
use crate::JSONSchema;
use serde_json::{Map, Value};

pub struct ContainsValidator<'a> {
    validators: Validators<'a>,
}

impl<'a> ContainsValidator<'a> {
    pub(crate) fn compile(
        schema: &'a Value,
        context: &CompilationContext,
    ) -> CompilationResult<'a> {
        Ok(Box::new(ContainsValidator {
            validators: compile_validators(schema, context)?,
        }))
    }
}

impl<'a> Validate<'a> for ContainsValidator<'a> {
    fn validate(&self, config: &JSONSchema, instance: &Value) -> ValidationResult {
        if let Value::Array(items) = instance {
            for item in items {
                if self
                    .validators
                    .iter()
                    .all(|validator| validator.validate(config, item).is_ok())
                {
                    return Ok(());
                }
            }
            return Err(ValidationError::contains(instance.clone()));
        }
        Ok(())
    }
    fn name(&self) -> String {
        format!("<contains: {:?}>", self.validators)
    }
}

pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    Some(ContainsValidator::compile(schema, context))
}
