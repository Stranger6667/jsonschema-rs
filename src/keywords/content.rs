use super::Validate;
use super::{CompilationResult, ValidationResult};
use crate::context::CompilationContext;
use crate::error::{CompilationError, ValidationError};
use crate::JSONSchema;
use serde_json::{from_str, Map, Value};

pub struct ContentMediaTypeValidator {
    func: fn(&str) -> ValidationResult,
}

impl<'a> ContentMediaTypeValidator {
    pub(crate) fn compile(func: fn(&str) -> ValidationResult) -> CompilationResult<'a> {
        Ok(Box::new(ContentMediaTypeValidator { func }))
    }
}

impl<'a> Validate<'a> for ContentMediaTypeValidator {
    fn validate(&self, _: &JSONSchema, instance: &Value) -> ValidationResult {
        if let Value::String(item) = instance {
            return (self.func)(item);
        }
        Ok(())
    }
    fn name(&self) -> String {
        // TODO. store media type
        "<content media type: TODO>".to_string()
    }
}

pub struct ContentEncodingValidator {
    func: fn(&str) -> ValidationResult,
}

impl<'a> ContentEncodingValidator {
    pub(crate) fn compile(func: fn(&str) -> ValidationResult) -> CompilationResult<'a> {
        Ok(Box::new(ContentEncodingValidator { func }))
    }
}

impl<'a> Validate<'a> for ContentEncodingValidator {
    fn validate(&self, _: &JSONSchema, instance: &Value) -> ValidationResult {
        if let Value::String(item) = instance {
            return (self.func)(item);
        }
        Ok(())
    }
    fn name(&self) -> String {
        // TODO. store encoding
        "<content encoding: TODO>".to_string()
    }
}

pub struct ContentMediaTypeAndEncodingValidator {
    func: fn(&str) -> ValidationResult,
    converter: fn(&str) -> Result<String, ValidationError>,
}

impl<'a> ContentMediaTypeAndEncodingValidator {
    pub(crate) fn compile(
        func: fn(&str) -> ValidationResult,
        converter: fn(&str) -> Result<String, ValidationError>,
    ) -> CompilationResult<'a> {
        Ok(Box::new(ContentMediaTypeAndEncodingValidator {
            func,
            converter,
        }))
    }
}

impl<'a> Validate<'a> for ContentMediaTypeAndEncodingValidator {
    fn validate(&self, _: &JSONSchema, instance: &Value) -> ValidationResult {
        if let Value::String(item) = instance {
            let converted = (self.converter)(item)?;
            return (self.func)(&converted);
        }
        Ok(())
    }
    fn name(&self) -> String {
        // TODO. store encoding
        "<content media type & encoding: TODO>".to_string()
    }
}

pub(crate) fn is_json(instance: &str) -> ValidationResult {
    if from_str::<Value>(instance).is_err() {
        return Err(ValidationError::format(
            instance.to_owned(),
            "application/json",
        ));
    }
    Ok(())
}

pub(crate) fn is_base64(instance: &str) -> ValidationResult {
    if base64::decode(instance).is_err() {
        return Err(ValidationError::format(instance.to_owned(), "base64"));
    }
    Ok(())
}

pub(crate) fn from_base64(instance: &str) -> Result<String, ValidationError> {
    match base64::decode(instance) {
        Ok(value) => Ok(String::from_utf8(value)?),
        Err(_) => Err(ValidationError::format(instance.to_owned(), "base64")),
    }
}

pub(crate) fn compile_media_type<'a>(
    schema: &'a Map<String, Value>,
    subschema: &'a Value,
    _: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    match subschema {
        Value::String(content_type) => {
            let func = match content_type.as_str() {
                "application/json" => is_json,
                _ => return None,
            };
            if let Some(content_encoding) = schema.get("contentEncoding") {
                match content_encoding {
                    Value::String(content_encoding) => {
                        let converter = match content_encoding.as_str() {
                            "base64" => from_base64,
                            _ => return None,
                        };
                        Some(ContentMediaTypeAndEncodingValidator::compile(
                            func, converter,
                        ))
                    }
                    _ => Some(Err(CompilationError::SchemaError)),
                }
            } else {
                Some(ContentMediaTypeValidator::compile(func))
            }
        }
        _ => Some(Err(CompilationError::SchemaError)),
    }
}

pub(crate) fn compile_content_encoding<'a>(
    schema: &'a Map<String, Value>,
    subschema: &'a Value,
    _: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    // Performed during media type validation
    if schema.get("contentMediaType").is_some() {
        // TODO. what if media type is not supported?
        return None;
    }
    match subschema {
        Value::String(content_encoding) => {
            let func = match content_encoding.as_str() {
                "base64" => is_base64,
                _ => return None,
            };
            Some(ContentEncodingValidator::compile(func))
        }
        _ => Some(Err(CompilationError::SchemaError)),
    }
}
