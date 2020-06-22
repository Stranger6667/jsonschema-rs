//! Validators for `contentEncoding` keywords.
use crate::{
    compilation::{CompilationContext, JSONSchema},
    error::{error, no_error, CompilationError, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    validator::Validate,
};
use serde_json::{Map, Value};

/// Validator for `contentEncoding` keyword.
pub struct ContentEncodingValidator {
    encoding: String,
    func: for<'a> fn(&'a Value, &str) -> ErrorIterator<'a>,
}

impl ContentEncodingValidator {
    #[inline]
    pub(crate) fn compile(
        encoding: &str,
        func: for<'a> fn(&'a Value, &str) -> ErrorIterator<'a>,
    ) -> CompilationResult {
        Ok(Box::new(ContentEncodingValidator {
            encoding: encoding.to_string(),
            func,
        }))
    }
}

impl Validate for ContentEncodingValidator {
    fn name(&self) -> String {
        format!("contentEncoding: {}", self.encoding)
    }

    #[inline]
    fn is_valid_string(&self, _: &JSONSchema, instance: &Value, instance_value: &str) -> bool {
        (self.func)(instance, instance_value).next().is_none()
    }
    #[inline]
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(instance_value) = instance {
            self.is_valid_string(schema, instance, instance_value)
        } else {
            true
        }
    }

    #[inline]
    fn validate_string<'a>(
        &self,
        _: &'a JSONSchema,
        instance: &'a Value,
        instance_value: &'a str,
    ) -> ErrorIterator<'a> {
        (self.func)(instance, instance_value)
    }
    #[inline]
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::String(instance_value) = instance {
            self.validate_string(schema, instance, instance_value)
        } else {
            no_error()
        }
    }
}

pub fn is_base64<'a>(instance: &'a Value, instance_string: &str) -> ErrorIterator<'a> {
    if base64::decode(instance_string).is_err() {
        return error(ValidationError::format(instance, "base64"));
    }
    no_error()
}

#[inline]
pub fn compile(
    _: &Map<String, Value>,
    subschema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    match subschema {
        Value::String(content_encoding) => {
            let func = match content_encoding.as_str() {
                "base64" => is_base64,
                _ => return None,
            };
            Some(ContentEncodingValidator::compile(content_encoding, func))
        }
        _ => Some(Err(CompilationError::SchemaError)),
    }
}
