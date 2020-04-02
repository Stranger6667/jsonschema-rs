use super::{CompilationResult, Validate, Validators};
use crate::compilation::{compile_validators, CompilationContext, JSONSchema};
use crate::error::{no_error, CompilationError, ErrorIterator, ValidationError};
use serde_json::{Map, Value};

pub struct AnyOfValidator {
    schemas: Vec<Validators>,
}

impl AnyOfValidator {
    pub(crate) fn compile(schema: &Value, context: &CompilationContext) -> CompilationResult {
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

impl Validate for AnyOfValidator {
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if !self.is_valid(schema, instance) {
            return ValidationError::any_of(instance.clone());
        }
        no_error()
    }

    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        for validators in self.schemas.iter() {
            if validators
                .iter()
                .all(|validator| validator.is_valid(schema, instance))
            {
                return true;
            }
        }
        false
    }

    fn name(&self) -> String {
        format!("<any of: {:?}>", self.schemas)
    }
}

pub(crate) fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    Some(AnyOfValidator::compile(schema, context))
}
