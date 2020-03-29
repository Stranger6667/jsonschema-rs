use super::CompilationResult;
use super::{Validate, Validators};
use crate::compilation::compile_validators;
use crate::compilation::CompilationContext;
use crate::error::{CompilationError, ErrorIterator};
use crate::JSONSchema;
use serde_json::{Map, Value};

pub struct AllOfValidator {
    schemas: Vec<Validators>,
}

impl AllOfValidator {
    pub(crate) fn compile(schema: &Value, context: &CompilationContext) -> CompilationResult {
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

impl Validate for AllOfValidator {
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        let errors: Vec<_> = self
            .schemas
            .iter()
            .flat_map(move |validators| {
                validators
                    .iter()
                    .flat_map(move |validator| validator.validate(schema, instance))
            })
            .collect();
        Box::new(errors.into_iter())
    }
    fn name(&self) -> String {
        format!("<all of: {:?}>", self.schemas)
    }
}
pub(crate) fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    Some(AllOfValidator::compile(schema, context))
}
