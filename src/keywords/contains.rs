use crate::{
    compilation::{compile_validators, context::CompilationContext, JSONSchema},
    error::{no_error, ErrorIterator, ValidationError},
    keywords::{format_validators, CompilationResult, Validators},
    validator::Validate,
};
use serde_json::{Map, Value};

pub struct ContainsValidator {
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
    #[inline]
    fn build_validation_error<'a>(&self, instance: &'a Value) -> ValidationError<'a> {
        ValidationError::contains(instance)
    }

    #[inline]
    fn is_valid_array(&self, schema: &JSONSchema, _: &Value, instance_value: &[Value]) -> bool {
        for item in instance_value {
            if self
                .validators
                .iter()
                .all(|validator| validator.is_valid(schema, item))
            {
                return true;
            }
        }
        false
    }
    #[inline]
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Array(instance_value) = instance {
            self.is_valid_array(schema, instance, instance_value)
        } else {
            true
        }
    }

    #[inline]
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Array(instance_value) = instance {
            self.validate_array(schema, instance, instance_value)
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
pub fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    Some(ContainsValidator::compile(schema, context))
}
