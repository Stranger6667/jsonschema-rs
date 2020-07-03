use crate::{
    compilation::{context::CompilationContext, JSONSchema},
    error::{error, no_error, CompilationError, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    validator::Validate,
};
use serde_json::{Map, Value};

pub(crate) struct RequiredValidator {
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
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(instance_value) = instance {
            self.is_valid_object(schema, instance, instance_value)
        } else {
            true
        }
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
    #[inline]
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Object(instance_value) = instance {
            self.validate_object(schema, instance, instance_value)
        } else {
            no_error()
        }
    }
}
impl ToString for RequiredValidator {
    fn to_string(&self) -> String {
        format!("required: [{}]", self.required.join(", "))
    }
}

#[inline]
pub(crate) fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    Some(RequiredValidator::compile(schema))
}
