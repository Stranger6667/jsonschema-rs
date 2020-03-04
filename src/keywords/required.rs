use super::Validate;
use super::{CompilationResult, ValidationResult};
use crate::context::CompilationContext;
use crate::error::{CompilationError, ValidationError};
use crate::JSONSchema;
use serde_json::{Map, Value};

pub struct RequiredValidator<'a> {
    required: Vec<&'a String>,
}

impl<'a> RequiredValidator<'a> {
    pub(crate) fn compile(schema: &'a Value) -> CompilationResult<'a> {
        match schema {
            Value::Array(items) => {
                let mut required = Vec::with_capacity(items.len());
                for item in items {
                    match item {
                        Value::String(string) => required.push(string),
                        _ => return Err(CompilationError::SchemaError),
                    }
                }
                Ok(Box::new(RequiredValidator { required }))
            }
            _ => Err(CompilationError::SchemaError),
        }
    }
}

impl<'a> Validate<'a> for RequiredValidator<'a> {
    fn validate(&self, _: &JSONSchema, instance: &Value) -> ValidationResult {
        if let Value::Object(item) = instance {
            for property_name in self.required.iter() {
                let name = *property_name;
                if !item.contains_key(name) {
                    return Err(ValidationError::required(name.clone()));
                }
            }
        }
        Ok(())
    }

    fn name(&self) -> String {
        format!("<required: {:?}>", self.required)
    }
}

pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    _: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    Some(RequiredValidator::compile(schema))
}
