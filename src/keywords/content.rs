use super::Validate;
use super::{CompilationResult, ErrorIterator};
use crate::context::CompilationContext;
use crate::error::{error, no_error, CompilationError, ValidationError};
use crate::JSONSchema;
use serde_json::{from_str, Map, Value};

pub struct ContentMediaTypeValidator {
    func: fn(&str) -> ErrorIterator,
}

impl ContentMediaTypeValidator {
    pub(crate) fn compile(func: fn(&str) -> ErrorIterator) -> CompilationResult {
        Ok(Box::new(ContentMediaTypeValidator { func }))
    }
}

impl Validate for ContentMediaTypeValidator {
    fn validate<'a>(&self, _: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::String(item) = instance {
            return (self.func)(item);
        }
        no_error()
    }
    fn name(&self) -> String {
        // TODO. store media type
        "<content media type: TODO>".to_string()
    }
}

pub struct ContentEncodingValidator {
    func: fn(&str) -> ErrorIterator,
}

impl ContentEncodingValidator {
    pub(crate) fn compile(func: fn(&str) -> ErrorIterator) -> CompilationResult {
        Ok(Box::new(ContentEncodingValidator { func }))
    }
}

impl Validate for ContentEncodingValidator {
    fn validate<'a>(&self, _: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::String(item) = instance {
            return (self.func)(item);
        }
        no_error()
    }
    fn name(&self) -> String {
        // TODO. store encoding
        "<content encoding: TODO>".to_string()
    }
}

pub struct ContentMediaTypeAndEncodingValidator {
    func: fn(&str) -> ErrorIterator,
    converter: fn(&str) -> Result<String, ValidationError>,
}

impl<'a> ContentMediaTypeAndEncodingValidator {
    pub(crate) fn compile(
        func: fn(&str) -> ErrorIterator,
        converter: fn(&str) -> Result<String, ValidationError>,
    ) -> CompilationResult {
        Ok(Box::new(ContentMediaTypeAndEncodingValidator {
            func,
            converter,
        }))
    }
}

impl Validate for ContentMediaTypeAndEncodingValidator {
    fn validate<'a>(&self, _: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::String(item) = instance {
            return match (self.converter)(item) {
                Ok(converted) => {
                    let errors: Vec<_> = (self.func)(&converted).collect();
                    Box::new(errors.into_iter())
                }
                Err(e) => error(e),
            };
        }
        no_error()
    }
    fn name(&self) -> String {
        // TODO. store encoding
        "<content media type & encoding: TODO>".to_string()
    }
}

pub(crate) fn is_json(instance: &str) -> ErrorIterator {
    if from_str::<Value>(instance).is_err() {
        return error(ValidationError::format(
            instance.to_owned(),
            "application/json",
        ));
    }
    no_error()
}

pub(crate) fn is_base64(instance: &str) -> ErrorIterator {
    if base64::decode(instance).is_err() {
        return error(ValidationError::format(instance.to_owned(), "base64"));
    }
    no_error()
}

pub(crate) fn from_base64(instance: &str) -> Result<String, ValidationError> {
    match base64::decode(instance) {
        Ok(value) => Ok(String::from_utf8(value)?),
        Err(_) => Err(ValidationError::format(instance.to_owned(), "base64")),
    }
}

pub(crate) fn compile_media_type(
    schema: &Map<String, Value>,
    subschema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
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

pub(crate) fn compile_content_encoding(
    schema: &Map<String, Value>,
    subschema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
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
