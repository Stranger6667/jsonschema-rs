use super::{CompilationResult, Validate, Validators};
use crate::{
    compilation::{compile_validators, CompilationContext, JSONSchema},
    error::{error, no_error, CompilationError, ErrorIterator, ValidationError},
};
use serde_json::{Map, Value};

pub struct AnyOfValidator {
    schemas: Vec<Validators>,
}

impl AnyOfValidator {
    #[inline]
    pub(crate) fn compile(schema: &Value, context: &CompilationContext) -> CompilationResult {
        if let Value::Array(items) = schema {
            let mut schemas = Vec::with_capacity(items.len());
            for item in items {
                let validators = compile_validators(item, context)?;
                schemas.push(validators)
            }
            return Ok(Box::new(AnyOfValidator { schemas }));
        }
        Err(CompilationError::SchemaError)
    }
}

impl Validate for AnyOfValidator {
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::any_of(instance))
        }
    }

    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        for validators in &self.schemas {
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

#[inline]
pub fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    Some(AnyOfValidator::compile(schema, context))
}
