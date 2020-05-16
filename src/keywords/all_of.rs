use super::{CompilationResult, Validate, Validators};
use crate::{
    compilation::{compile_validators, CompilationContext, JSONSchema},
    error::{CompilationError, ErrorIterator},
};
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

    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        self.schemas.iter().all(move |validators| {
            validators
                .iter()
                .all(move |validator| validator.is_valid(schema, instance))
        })
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
