use super::{CompilationResult, ValidationResult};
use super::{Validate, Validators};
use crate::context::CompilationContext;
use crate::error::CompilationError;
use crate::validator::compile_validators;
use crate::JSONSchema;
use serde_json::{Map, Value};

pub struct AllOfValidator<'a> {
    schemas: Vec<Validators<'a>>,
}

impl<'a> AllOfValidator<'a> {
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
                Ok(Box::new(AllOfValidator { schemas }))
            }
            None => Err(CompilationError::SchemaError),
        }
    }
}

impl<'a> Validate<'a> for AllOfValidator<'a> {
    fn validate(&self, config: &JSONSchema, instance: &Value) -> ValidationResult {
        for validators in self.schemas.iter() {
            for validator in validators {
                validator.validate(config, instance)?
            }
        }
        Ok(())
    }
    fn name(&self) -> String {
        format!("<all of: {:?}>", self.schemas)
    }
}
pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    Some(AllOfValidator::compile(schema, context))
}
