use crate::keywords::InstancePath;
use crate::{
    compilation::{compile_validators, context::CompilationContext, JSONSchema},
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::{format_validators, CompilationResult, Validators},
    validator::Validate,
};
use serde_json::{Map, Value};

pub(crate) struct ContainsValidator {
    validators: Validators,
}

impl ContainsValidator {
    #[inline]
    pub(crate) fn compile(schema: &Value, context: &CompilationContext) -> CompilationResult {
        Ok(Box::new(ContainsValidator {
            validators: compile_validators(schema, context)?,
        }))
    }
}

impl Validate for ContainsValidator {
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Array(items) = instance {
            for item in items {
                if self
                    .validators
                    .iter()
                    .all(|validator| validator.is_valid(schema, item))
                {
                    return true;
                }
            }
            false
        } else {
            true
        }
    }

    fn validate<'a, 'b>(
        &'b self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        curr_instance_path: InstancePath<'b>,
    ) -> ErrorIterator<'a> {
        if let Value::Array(items) = instance {
            for item in items {
                if self
                    .validators
                    .iter()
                    .all(|validator| validator.is_valid(schema, item))
                {
                    return no_error();
                }
            }
            error(ValidationError::contains(
                curr_instance_path.into(),
                instance,
            ))
        } else {
            no_error()
        }
    }
}

impl ToString for ContainsValidator {
    fn to_string(&self) -> String {
        format!("contains: {}", format_validators(&self.validators))
    }
}

#[inline]
pub(crate) fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    Some(ContainsValidator::compile(schema, context))
}
