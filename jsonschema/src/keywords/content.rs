//! Validators for `contentMediaType` and `contentEncoding` keywords.
use crate::{
    compilation::{context::CompilationContext, JSONSchema},
    content_encoding::{ContentEncodingCheckType, ContentEncodingConverterType},
    content_media_type::ContentMediaTypeCheckType,
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    paths::{InstancePath, JSONPointer},
    validator::Validate,
};
use serde_json::{Map, Value};

/// Validator for `contentMediaType` keyword.
pub(crate) struct ContentMediaTypeValidator {
    media_type: String,
    func: ContentMediaTypeCheckType,
    schema_path: JSONPointer,
}

impl ContentMediaTypeValidator {
    #[inline]
    pub(crate) fn compile(
        media_type: &str,
        func: ContentMediaTypeCheckType,
        schema_path: JSONPointer,
    ) -> CompilationResult {
        Ok(Box::new(ContentMediaTypeValidator {
            media_type: media_type.to_string(),
            func,
            schema_path,
        }))
    }
}

/// Validator delegates validation to the stored function.
impl Validate for ContentMediaTypeValidator {
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            (self.func)(item)
        } else {
            true
        }
    }

    fn validate<'a>(
        &self,
        _: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if let Value::String(item) = instance {
            if (self.func)(item) {
                no_error()
            } else {
                error(ValidationError::content_media_type(
                    self.schema_path.clone(),
                    instance_path.into(),
                    instance,
                    &self.media_type,
                ))
            }
        } else {
            no_error()
        }
    }
}

impl ToString for ContentMediaTypeValidator {
    fn to_string(&self) -> String {
        format!("contentMediaType: {}", self.media_type)
    }
}

/// Validator for `contentEncoding` keyword.
pub(crate) struct ContentEncodingValidator {
    encoding: String,
    func: ContentEncodingCheckType,
    schema_path: JSONPointer,
}

impl ContentEncodingValidator {
    #[inline]
    pub(crate) fn compile(
        encoding: &str,
        func: ContentEncodingCheckType,
        schema_path: JSONPointer,
    ) -> CompilationResult {
        Ok(Box::new(ContentEncodingValidator {
            encoding: encoding.to_string(),
            func,
            schema_path,
        }))
    }
}

impl Validate for ContentEncodingValidator {
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            (self.func)(item)
        } else {
            true
        }
    }

    fn validate<'a>(
        &self,
        _: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if let Value::String(item) = instance {
            if (self.func)(item) {
                no_error()
            } else {
                error(ValidationError::content_encoding(
                    self.schema_path.clone(),
                    instance_path.into(),
                    instance,
                    &self.encoding,
                ))
            }
        } else {
            no_error()
        }
    }
}

impl ToString for ContentEncodingValidator {
    fn to_string(&self) -> String {
        format!("contentEncoding: {}", self.encoding)
    }
}

/// Combined validator for both `contentEncoding` and `contentMediaType` keywords.
pub(crate) struct ContentMediaTypeAndEncodingValidator {
    media_type: String,
    encoding: String,
    func: ContentMediaTypeCheckType,
    converter: ContentEncodingConverterType,
    schema_path: JSONPointer,
}

impl ContentMediaTypeAndEncodingValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        media_type: &'a str,
        encoding: &'a str,
        func: ContentMediaTypeCheckType,
        converter: ContentEncodingConverterType,
        schema_path: JSONPointer,
    ) -> CompilationResult<'a> {
        Ok(Box::new(ContentMediaTypeAndEncodingValidator {
            media_type: media_type.to_string(),
            encoding: encoding.to_string(),
            func,
            converter,
            schema_path,
        }))
    }
}

/// Decode the input value & check media type
impl Validate for ContentMediaTypeAndEncodingValidator {
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            match (self.converter)(item) {
                Ok(None) | Err(_) => false,
                Ok(Some(converted)) => (self.func)(&converted),
            }
        } else {
            true
        }
    }

    fn validate<'a>(
        &self,
        _: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if let Value::String(item) = instance {
            match (self.converter)(item) {
                Ok(None) => error(ValidationError::content_encoding(
                    self.schema_path.clone_with("contentEncoding"),
                    instance_path.into(),
                    instance,
                    &self.encoding,
                )),
                Ok(Some(converted)) => {
                    if (self.func)(&converted) {
                        no_error()
                    } else {
                        error(ValidationError::content_media_type(
                            self.schema_path.clone_with("contentMediaType"),
                            instance_path.into(),
                            instance,
                            &self.media_type,
                        ))
                    }
                }
                Err(e) => error(e),
            }
        } else {
            no_error()
        }
    }
}

impl ToString for ContentMediaTypeAndEncodingValidator {
    fn to_string(&self) -> String {
        format!(
            "{{contentMediaType: {}, contentEncoding: {}}}",
            self.media_type, self.encoding
        )
    }
}

#[inline]
pub(crate) fn compile_media_type<'a>(
    schema: &'a Map<String, Value>,
    subschema: &'a Value,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    match subschema {
        Value::String(media_type) => {
            let func = match context.config.content_media_type_check(media_type.as_str()) {
                Some(f) => f,
                None => return None,
            };
            if let Some(content_encoding) = schema.get("contentEncoding") {
                match content_encoding {
                    Value::String(content_encoding) => {
                        let converter = match context
                            .config
                            .content_encoding_convert(content_encoding.as_str())
                        {
                            Some(f) => f,
                            None => return None,
                        };
                        Some(ContentMediaTypeAndEncodingValidator::compile(
                            media_type,
                            content_encoding,
                            func,
                            converter,
                            context.schema_path.clone().into(),
                        ))
                    }
                    _ => Some(Err(ValidationError::schema(subschema))),
                }
            } else {
                Some(ContentMediaTypeValidator::compile(
                    media_type,
                    func,
                    context.as_pointer_with("contentMediaType"),
                ))
            }
        }
        _ => Some(Err(ValidationError::schema(subschema))),
    }
}

#[inline]
pub(crate) fn compile_content_encoding<'a>(
    schema: &'a Map<String, Value>,
    subschema: &'a Value,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    // Performed during media type validation
    if schema.get("contentMediaType").is_some() {
        // TODO. what if media type is not supported?
        return None;
    }
    match subschema {
        Value::String(content_encoding) => {
            let func = match context
                .config
                .content_encoding_check(content_encoding.as_str())
            {
                Some(f) => f,
                None => return None,
            };
            Some(ContentEncodingValidator::compile(
                content_encoding,
                func,
                context.as_pointer_with("contentEncoding"),
            ))
        }
        _ => Some(Err(ValidationError::schema(subschema))),
    }
}

#[cfg(test)]
mod tests {
    use crate::tests_util;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(&json!({"contentEncoding": "base64"}), &json!("asd"), "/contentEncoding")]
    #[test_case(&json!({"contentMediaType": "application/json"}), &json!("asd"), "/contentMediaType")]
    #[test_case(&json!({"contentMediaType": "application/json", "contentEncoding": "base64"}), &json!("ezp9Cg=="), "/contentMediaType")]
    #[test_case(&json!({"contentMediaType": "application/json", "contentEncoding": "base64"}), &json!("{}"), "/contentEncoding")]
    fn schema_path(schema: &Value, instance: &Value, expected: &str) {
        tests_util::assert_schema_path(schema, instance, expected)
    }
}
