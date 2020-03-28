use super::CompilationResult;
use super::Validate;
use crate::context::CompilationContext;
use crate::error::{no_error, CompilationError, ErrorIterator, ValidationError};
use crate::{helpers, JSONSchema};
use serde_json::{Map, Value};

pub struct EnumValidator {
    options: Value,
    items: Vec<Value>,
}

impl EnumValidator {
    pub(crate) fn compile(schema: &Value) -> CompilationResult {
        if let Value::Array(items) = schema {
            return Ok(Box::new(EnumValidator {
                options: schema.clone(),
                items: items.clone(),
            }));
        }
        Err(CompilationError::SchemaError)
    }
}

impl Validate for EnumValidator {
    fn validate<'a>(&self, _: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if !self.items.iter().any(|item| helpers::equal(instance, item)) {
            return ValidationError::enumeration(instance.clone(), self.options.clone());
        }
        no_error()
    }
    fn name(&self) -> String {
        format!("<enum: {:?}>", self.items)
    }
}

pub(crate) fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    Some(EnumValidator::compile(schema))
}
