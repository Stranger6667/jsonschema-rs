use crate::keywords::InstancePath;
use crate::{
    compilation::{compile_validators, context::CompilationContext, JSONSchema},
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::{format_validators, CompilationResult, Validators},
    validator::Validate,
};
use serde_json::{Map, Value};

pub(crate) struct NotValidator {
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
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        !self
            .validators
            .iter()
            .all(|validator| validator.is_valid(schema, instance))
    }

    fn validate<'a, 'b>(
        &'b self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        curr_instance_path: InstancePath<'b>,
    ) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::not(
                curr_instance_path.into(),
                instance,
                self.original.clone(),
            ))
        }
    }
}

impl ToString for NotValidator {
    fn to_string(&self) -> String {
        format!("not: {}", format_validators(&self.validators))
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
