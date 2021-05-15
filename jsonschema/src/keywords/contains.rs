use crate::{
    compilation::{compile_validators, context::CompilationContext, JSONSchema},
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::{format_validators, ValidationResult, Validators},
    paths::InstancePath,
    validator::Validate,
};
use serde_json::{Map, Value};

pub(crate) struct ContainsValidator {
    validators: Validators,
}

impl ContainsValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        schema: &'a Value,
        context: &'a CompilationContext,
    ) -> ValidationResult<'a> {
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

    fn validate<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
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
            error(ValidationError::contains(instance_path.into(), instance))
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
pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    context: &'a CompilationContext,
) -> Option<ValidationResult<'a>> {
    Some(ContainsValidator::compile(schema, context))
}
