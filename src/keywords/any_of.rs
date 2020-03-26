use super::{CompilationResult, ValidationResult};
use super::{Validate, Validators};
use crate::context::CompilationContext;
use crate::error::{CompilationError, ValidationError};
use crate::validator::compile_validators;
use crate::JSONSchema;
use serde_json::{Map, Value};

pub struct AnyOfValidator<'a> {
    schemas: Vec<Validators<'a>>,
}

impl<'a> AnyOfValidator<'a> {
    pub(crate) fn compile(
        schema: &'a Value,
        context: &CompilationContext,
    ) -> CompilationResult<'a> {
        match schema.as_array() {
            Some(items) => {
                let mut schemas = Vec::with_capacity(items.len());
                for item in items {
                    let validators = compile_validators(item, context)?;
                    schemas.push(validators)
                }
                Ok(Box::new(AnyOfValidator { schemas }))
            }
            None => Err(CompilationError::SchemaError),
        }
    }
}

impl<'a> Validate<'a> for AnyOfValidator<'a> {
    fn validate(&self, schema: &JSONSchema, instance: &Value) -> ValidationResult {
        for validators in self.schemas.iter() {
            if validators
                .iter()
                .all(|validator| validator.is_valid(schema, instance))
            {
                return Ok(());
            }
        }
        Err(ValidationError::any_of(instance.clone()))
    }
    fn name(&self) -> String {
        format!("<any of: {:?}>", self.schemas)
    }
}

pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    Some(AnyOfValidator::compile(schema, context))
}
