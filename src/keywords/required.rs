use super::{CompilationResult, Validate};
use crate::{
    compilation::{CompilationContext, JSONSchema},
    error::{error, no_error, CompilationError, ErrorIterator, ValidationError},
};
use serde_json::{Map, Value};

pub struct RequiredValidator {
    required: Vec<String>,
}

impl RequiredValidator {
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
    fn validate<'a>(&self, _: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Object(item) = instance {
            for property_name in self.required.iter() {
                if !item.contains_key(property_name) {
                    return error(ValidationError::required(instance, property_name.clone()));
                }
            }
        }
        no_error()
    }

    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            return self
                .required
                .iter()
                .all(|property_name| item.contains_key(property_name));
        }
        true
    }

    fn name(&self) -> String {
        format!("<required: {:?}>", self.required)
    }
}

pub(crate) fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    Some(RequiredValidator::compile(schema))
}
