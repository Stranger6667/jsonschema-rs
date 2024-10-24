//! Validators for `contentMediaType` and `contentEncoding` keywords.
use crate::{
    compiler,
    content_encoding::{ContentEncodingCheckType, ContentEncodingConverterType},
    content_media_type::ContentMediaTypeCheckType,
    error::ValidationError,
    keywords::CompilationResult,
    paths::{LazyLocation, Location},
    primitive_type::PrimitiveType,
    validator::Validate,
};
use serde_json::{Map, Value};

/// Validator for `contentMediaType` keyword.
pub(crate) struct ContentMediaTypeValidator {
    media_type: String,
    func: ContentMediaTypeCheckType,
    location: Location,
}

impl ContentMediaTypeValidator {
    #[inline]
    pub(crate) fn compile(
        media_type: &str,
        func: ContentMediaTypeCheckType,
        location: Location,
    ) -> CompilationResult {
        Ok(Box::new(ContentMediaTypeValidator {
            media_type: media_type.to_string(),
            func,
            location,
        }))
    }
}

/// Validator delegates validation to the stored function.
impl Validate for ContentMediaTypeValidator {
    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            (self.func)(item)
        } else {
            true
        }
    }

    fn validate<'i>(
        &self,
        instance: &'i Value,
        location: &LazyLocation,
    ) -> Result<(), ValidationError<'i>> {
        if let Value::String(item) = instance {
            if (self.func)(item) {
                Ok(())
            } else {
                Err(ValidationError::content_media_type(
                    self.location.clone(),
                    location.into(),
                    instance,
                    &self.media_type,
                ))
            }
        } else {
            Ok(())
        }
    }
}

/// Validator for `contentEncoding` keyword.
pub(crate) struct ContentEncodingValidator {
    encoding: String,
    func: ContentEncodingCheckType,
    location: Location,
}

impl ContentEncodingValidator {
    #[inline]
    pub(crate) fn compile(
        encoding: &str,
        func: ContentEncodingCheckType,
        location: Location,
    ) -> CompilationResult {
        Ok(Box::new(ContentEncodingValidator {
            encoding: encoding.to_string(),
            func,
            location,
        }))
    }
}

impl Validate for ContentEncodingValidator {
    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            (self.func)(item)
        } else {
            true
        }
    }

    fn validate<'i>(
        &self,
        instance: &'i Value,
        location: &LazyLocation,
    ) -> Result<(), ValidationError<'i>> {
        if let Value::String(item) = instance {
            if (self.func)(item) {
                Ok(())
            } else {
                Err(ValidationError::content_encoding(
                    self.location.clone(),
                    location.into(),
                    instance,
                    &self.encoding,
                ))
            }
        } else {
            Ok(())
        }
    }
}

/// Combined validator for both `contentEncoding` and `contentMediaType` keywords.
pub(crate) struct ContentMediaTypeAndEncodingValidator {
    media_type: String,
    encoding: String,
    func: ContentMediaTypeCheckType,
    converter: ContentEncodingConverterType,
    location: Location,
}

impl ContentMediaTypeAndEncodingValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        media_type: &'a str,
        encoding: &'a str,
        func: ContentMediaTypeCheckType,
        converter: ContentEncodingConverterType,
        location: Location,
    ) -> CompilationResult<'a> {
        Ok(Box::new(ContentMediaTypeAndEncodingValidator {
            media_type: media_type.to_string(),
            encoding: encoding.to_string(),
            func,
            converter,
            location,
        }))
    }
}

/// Decode the input value & check media type
impl Validate for ContentMediaTypeAndEncodingValidator {
    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            match (self.converter)(item) {
                Ok(None) | Err(_) => false,
                Ok(Some(converted)) => (self.func)(&converted),
            }
        } else {
            true
        }
    }

    fn validate<'i>(
        &self,
        instance: &'i Value,
        location: &LazyLocation,
    ) -> Result<(), ValidationError<'i>> {
        if let Value::String(item) = instance {
            match (self.converter)(item) {
                Ok(None) => Err(ValidationError::content_encoding(
                    self.location.join("contentEncoding"),
                    location.into(),
                    instance,
                    &self.encoding,
                )),
                Ok(Some(converted)) => {
                    if (self.func)(&converted) {
                        Ok(())
                    } else {
                        Err(ValidationError::content_media_type(
                            self.location.join("contentMediaType"),
                            location.into(),
                            instance,
                            &self.media_type,
                        ))
                    }
                }
                Err(e) => Err(e),
            }
        } else {
            Ok(())
        }
    }
}

#[inline]
pub(crate) fn compile_media_type<'a>(
    ctx: &compiler::Context,
    schema: &'a Map<String, Value>,
    subschema: &'a Value,
) -> Option<CompilationResult<'a>> {
    match subschema {
        Value::String(media_type) => {
            let func = match ctx.get_content_media_type_check(media_type.as_str()) {
                Some(f) => f,
                None => return None,
            };
            if let Some(content_encoding) = schema.get("contentEncoding") {
                match content_encoding {
                    Value::String(content_encoding) => {
                        let converter = match ctx.get_content_encoding_convert(content_encoding) {
                            Some(f) => f,
                            None => return None,
                        };
                        Some(ContentMediaTypeAndEncodingValidator::compile(
                            media_type,
                            content_encoding,
                            func,
                            converter,
                            ctx.location().clone(),
                        ))
                    }
                    _ => Some(Err(ValidationError::single_type_error(
                        Location::new(),
                        ctx.location().clone(),
                        content_encoding,
                        PrimitiveType::String,
                    ))),
                }
            } else {
                Some(ContentMediaTypeValidator::compile(
                    media_type,
                    func,
                    ctx.location().join("contentMediaType"),
                ))
            }
        }
        _ => Some(Err(ValidationError::single_type_error(
            Location::new(),
            ctx.location().clone(),
            subschema,
            PrimitiveType::String,
        ))),
    }
}

#[inline]
pub(crate) fn compile_content_encoding<'a>(
    ctx: &compiler::Context,
    schema: &'a Map<String, Value>,
    subschema: &'a Value,
) -> Option<CompilationResult<'a>> {
    // Performed during media type validation
    if schema.get("contentMediaType").is_some() {
        // TODO. what if media type is not supported?
        return None;
    }
    match subschema {
        Value::String(content_encoding) => {
            let func = match ctx.get_content_encoding_check(content_encoding) {
                Some(f) => f,
                None => return None,
            };
            Some(ContentEncodingValidator::compile(
                content_encoding,
                func,
                ctx.location().join("contentEncoding"),
            ))
        }
        _ => Some(Err(ValidationError::single_type_error(
            Location::new(),
            ctx.location().clone(),
            subschema,
            PrimitiveType::String,
        ))),
    }
}

#[cfg(test)]
mod tests {
    use referencing::Draft;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(&json!({"contentEncoding": "base64"}), &json!("asd"), "/contentEncoding")]
    #[test_case(&json!({"contentMediaType": "application/json"}), &json!("asd"), "/contentMediaType")]
    #[test_case(&json!({"contentMediaType": "application/json", "contentEncoding": "base64"}), &json!("ezp9Cg=="), "/contentMediaType")]
    #[test_case(&json!({"contentMediaType": "application/json", "contentEncoding": "base64"}), &json!("{}"), "/contentEncoding")]
    fn location(schema: &Value, instance: &Value, expected: &str) {
        let validator = crate::options()
            .with_draft(Draft::Draft7)
            .build(schema)
            .expect("Invalid schema");
        let error = validator.validate(instance).expect_err("Should fail");
        assert_eq!(error.schema_path.as_str(), expected);
    }
}
