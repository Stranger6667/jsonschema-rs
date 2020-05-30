//! Validators for `contentMediaType` and `contentEncoding` keywords.
use crate::{
    compilation::{CompilationContext, JSONSchema},
    error::{error, no_error, CompilationError, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    validator::Validate,
};
use serde_json::{from_str, Map, Value};

/// Validator for `contentMediaType` keyword.
pub struct ContentMediaTypeValidator {
    media_type: String,
    func: for<'a> fn(&'a Value, &str) -> ErrorIterator<'a>,
}

impl ContentMediaTypeValidator {
    #[inline]
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
    fn name(&self) -> String {
        format!("contentMediaType: {}", self.media_type)
    }

    #[inline]
    fn is_valid_string(&self, _: &JSONSchema, instance: &Value, instance_value: &str) -> bool {
        (self.func)(instance, instance_value).next().is_none()
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
}

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
    fn validate_string<'a>(
        &self,
        _: &'a JSONSchema,
        instance: &'a Value,
        instance_value: &'a str,
    ) -> ErrorIterator<'a> {
        (self.func)(instance, instance_value)
    }
}

/// Combined validator for both `contentEncoding` and `contentMediaType` keywords.
pub struct ContentMediaTypeAndEncodingValidator {
    media_type: String,
    encoding: String,
    func: for<'a> fn(&'a Value, &str) -> ErrorIterator<'a>,
    converter: for<'a> fn(&'a Value, &str) -> Result<String, ValidationError<'a>>,
}

impl ContentMediaTypeAndEncodingValidator {
    #[inline]
    pub(crate) fn compile(
        media_type: &str,
        encoding: &str,
        func: for<'a> fn(&'a Value, &str) -> ErrorIterator<'a>,
        converter: for<'a> fn(&'a Value, &str) -> Result<String, ValidationError<'a>>,
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
    fn name(&self) -> String {
        format!(
            "{{contentMediaType: {}, contentEncoding: {}}}",
            self.media_type, self.encoding
        )
    }

    #[inline]
    fn is_valid_string(&self, _: &JSONSchema, instance: &Value, instance_value: &str) -> bool {
        match (self.converter)(instance, instance_value) {
            Ok(converted) => (self.func)(instance, &converted).next().is_none(),
            Err(_) => false,
        }
    }

    #[inline]
    fn validate_string<'a>(
        &self,
        _: &'a JSONSchema,
        instance: &'a Value,
        instance_value: &'a str,
    ) -> ErrorIterator<'a> {
        // TODO. Avoid explicit `error` call. It might be done if `converter` will
        // return a proper type
        match (self.converter)(instance, instance_value) {
            Ok(converted) => {
                let errors: Vec<_> = (self.func)(instance, &converted).collect();
                Box::new(errors.into_iter())
            }
            Err(e) => error(e),
        }
    }
}

pub fn is_json<'a>(instance: &'a Value, instance_string: &str) -> ErrorIterator<'a> {
    if from_str::<Value>(instance_string).is_err() {
        return error(ValidationError::format(instance, "application/json"));
    }
    no_error()
}

pub fn is_base64<'a>(instance: &'a Value, instance_string: &str) -> ErrorIterator<'a> {
    if base64::decode(instance_string).is_err() {
        return error(ValidationError::format(instance, "base64"));
    }
    no_error()
}

pub fn from_base64<'a>(
    instance: &'a Value,
    instance_string: &str,
) -> Result<String, ValidationError<'a>> {
    match base64::decode(instance_string) {
        Ok(value) => Ok(String::from_utf8(value)?),
        Err(_) => Err(ValidationError::format(instance, "base64")),
    }
}

#[inline]
pub fn compile_media_type(
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

#[inline]
pub fn compile_content_encoding(
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
