use super::Validate;
use super::{CompilationResult, ValidationResult};
use crate::context::CompilationContext;
use crate::error::{CompilationError, ValidationError};
use crate::{helpers, JSONSchema};
use serde_json::{Map, Value};

pub struct EnumValidator<'a> {
    options: &'a Value,
    items: &'a Vec<Value>,
}

impl<'a> EnumValidator<'a> {
    pub(crate) fn compile(schema: &'a Value) -> CompilationResult<'a> {
        if let Value::Array(items) = schema {
            return Ok(Box::new(EnumValidator {
                options: schema,
                items,
            }));
        }
        Err(CompilationError::SchemaError)
    }
}

impl<'a> Validate<'a> for EnumValidator<'a> {
    fn validate(&self, _: &JSONSchema, instance: &Value) -> ValidationResult {
        if !self.items.iter().any(|item| helpers::equal(instance, item)) {
            return Err(ValidationError::enumeration(
                instance.clone(),
                self.options.clone(),
            ));
        }
        Ok(())
    }
    fn name(&self) -> String {
        format!("<enum: {:?}>", self.items)
    }
}

pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    _: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    Some(EnumValidator::compile(schema))
}
