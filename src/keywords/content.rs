//! Validators for `contentMediaType` and `contentEncoding` keywords.
use super::{CompilationResult, ErrorIterator, Validate};
use crate::{
    compilation::{CompilationContext, JSONSchema},
    error::{error, no_error, CompilationError, ValidationError},
};
use serde_json::{from_str, Map, Value};

/// Validator for `contentMediaType` keyword.
pub struct ContentMediaTypeValidator {
    media_type: String,
    func: for<'a> fn(&'a Value, &str) -> ErrorIterator<'a>,
}

impl ContentMediaTypeValidator {
    pub(crate) fn compile(
        media_type: &str,
        func: for<'a> fn(&'a Value, &str) -> ErrorIterator<'a>,
    ) -> CompilationResult {
        Ok(Box::new(ContentMediaTypeValidator {
            media_type: media_type.to_string(),
            func,
        }))
    }
}

/// Validator delegates validation to the stored function.
impl Validate for ContentMediaTypeValidator {
    fn validate<'a>(&self, _: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::String(item) = instance {
            return (self.func)(instance, item);
        }
        no_error()
    }

    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            return (self.func)(instance, item).next().is_none();
        }
        true
    }

    fn name(&self) -> String {
        format!("<contentMediaType: {}>", self.media_type)
    }
}

/// Validator for `contentEncoding` keyword.
pub struct ContentEncodingValidator {
    encoding: String,
    func: for<'a> fn(&'a Value, &str) -> ErrorIterator<'a>,
}

impl ContentEncodingValidator {
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
    fn validate<'a>(&self, _: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::String(item) = instance {
            return (self.func)(instance, item);
        }
        no_error()
    }

    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            return (self.func)(instance, item).next().is_none();
        }
        true
    }

    fn name(&self) -> String {
        format!("<contentEncoding: {}>", self.encoding)
    }
}

/// Combined validator for both `contentEncoding` and `contentMediaType` keywords.
pub struct ContentMediaTypeAndEncodingValidator {
    media_type: String,
    encoding: String,
    func: for<'a> fn(&'a Value, &str) -> ErrorIterator<'a>,
    converter: fn(&Value, &str) -> Result<String, ValidationError>,
}

impl ContentMediaTypeAndEncodingValidator {
    pub(crate) fn compile(
        media_type: &str,
        encoding: &str,
        func: for<'a> fn(&'a Value, &str) -> ErrorIterator<'a>,
        converter: fn(&Value, &str) -> Result<String, ValidationError>,
    ) -> CompilationResult {
        Ok(Box::new(ContentMediaTypeAndEncodingValidator {
            media_type: media_type.to_string(),
            encoding: encoding.to_string(),
            func,
            converter,
        }))
    }
}

/// Decode the input value & check media type
impl Validate for ContentMediaTypeAndEncodingValidator {
    fn validate<'a>(&self, _: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::String(item) = instance {
            // TODO. Avoid explicit `error` call. It might be done if `converter` will
            // return a proper type
            return match (self.converter)(instance, item) {
                Ok(converted) => {
                    let errors: Vec<_> = (self.func)(instance, &converted).collect();
                    Box::new(errors.into_iter())
                }
                Err(e) => error(e),
            };
        }
        no_error()
    }

    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            return match (self.converter)(instance, item) {
                Ok(converted) => (self.func)(instance, &converted).next().is_none(),
                Err(_) => false,
            };
        }
        true
    }

    fn name(&self) -> String {
        format!(
            "<contentMediaType - contentEncoding: {} - {}>",
            self.media_type, self.encoding
        )
    }
}

pub(crate) fn is_json<'a>(instance: &'a Value, instance_string: &str) -> ErrorIterator<'a> {
    if from_str::<Value>(instance_string).is_err() {
        return error(ValidationError::format(
            instance,
            "application/json".to_string(),
        ));
    }
    no_error()
}

pub(crate) fn is_base64<'a>(instance: &'a Value, instance_string: &str) -> ErrorIterator<'a> {
    if base64::decode(instance_string).is_err() {
        return error(ValidationError::format(instance, "base64".to_string()));
    }
    no_error()
}

pub(crate) fn from_base64(
    instance: &Value,
    instance_string: &str,
) -> Result<String, ValidationError> {
    match base64::decode(instance_string) {
        Ok(value) => Ok(String::from_utf8(value)?),
        Err(_) => Err(ValidationError::format(instance, "base64".to_string())),
    }
}

pub(crate) fn compile_media_type(
    schema: &Map<String, Value>,
    subschema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    match subschema {
        Value::String(media_type) => {
            let func = match media_type.as_str() {
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
                            media_type,
                            content_encoding,
                            func,
                            converter,
                        ))
                    }
                    _ => Some(Err(CompilationError::SchemaError)),
                }
            } else {
                Some(ContentMediaTypeValidator::compile(media_type, func))
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
            Some(ContentEncodingValidator::compile(content_encoding, func))
        }
        _ => Some(Err(CompilationError::SchemaError)),
    }
}
