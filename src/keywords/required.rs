use crate::{
    compilation::{CompilationContext, JSONSchema},
    error::{error, no_error, CompilationError, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    validator::Validate,
};
use serde_json::{Map, Value};

pub struct RequiredValidator {
    required: Vec<String>,
}

impl RequiredValidator {
    #[inline]
    pub(crate) fn compile(schema: &Value) -> CompilationResult {
        match schema {
            Value::Array(items) => {
                let mut required = Vec::with_capacity(items.len());
                for item in items {
                    match item {
                        Value::String(string) => required.push(string.clone()),
                        _ => return Err(CompilationError::SchemaError),
                    }
                }
                Ok(Box::new(RequiredValidator { required }))
            }
            _ => Err(CompilationError::SchemaError),
        }
    }
}

impl Validate for RequiredValidator {
    fn name(&self) -> String {
        format!("required: [{}]", self.required.join(", "))
    }

    #[inline]
    fn is_valid_object(
        &self,
        _: &JSONSchema,
        _: &Value,
        instance_value: &Map<String, Value>,
    ) -> bool {
        self.required
            .iter()
            .all(|property_name| instance_value.contains_key(property_name))
    }

    #[inline]
    fn validate_object<'a>(
        &self,
        _: &'a JSONSchema,
        instance: &'a Value,
        instance_value: &'a Map<String, Value>,
    ) -> ErrorIterator<'a> {
        for property_name in &self.required {
            if !instance_value.contains_key(property_name) {
                return error(ValidationError::required(instance, property_name.clone()));
            }
        }
        no_error()
    }
}

#[inline]
pub fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    Some(RequiredValidator::compile(schema))
}
