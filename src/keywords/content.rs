//! Validators for `contentMediaType` and `contentEncoding` keywords.
use crate::{
    compilation::{context::CompilationContext, JSONSchema},
    error::{error, no_error, CompilationError, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    validator::Validate,
};
use serde_json::{Map, Value};

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
impl ToString for ContentMediaTypeValidator {
    fn to_string(&self) -> String {
        format!("contentMediaType: {}", self.media_type)
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
impl ToString for ContentEncodingValidator {
    fn to_string(&self) -> String {
        format!("contentEncoding: {}", self.encoding)
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
    #[inline]
    fn is_valid_string(&self, _: &JSONSchema, instance: &Value, instance_value: &str) -> bool {
        match (self.converter)(instance, instance_value) {
            Ok(converted) => (self.func)(instance, &converted).next().is_none(),
            Err(_) => false,
        }
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
    #[inline]
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::String(instance_value) = instance {
            self.validate_string(schema, instance, instance_value)
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
pub fn compile_media_type(
    schema: &Map<String, Value>,
    subschema: &Value,
    context: &CompilationContext,
) -> Option<CompilationResult> {
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
    context: &CompilationContext,
) -> Option<CompilationResult> {
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
            Some(ContentEncodingValidator::compile(content_encoding, func))
        }
        _ => Some(Err(CompilationError::SchemaError)),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        error::{error, no_error, ErrorIterator, ValidationError},
        CompilationConfig, JSONSchema,
    };
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(&json!({"contentMediaType": "application/json"}), &json!("{}") => true)]
    #[test_case(&json!({"contentMediaType": "application/json"}), &json!("{") => false)]
    #[test_case(&json!({"contentMediaType": "test_media_type"}), &json!("whatever") => true)]
    #[test_case(&json!({"contentMediaType": "test_media_type"}), &json!("error") => false)]
    fn test_custom_content_media_type(schema: &Value, instance: &Value) -> bool {
        fn test_content_media_type_check<'a>(
            instance: &'a Value,
            instance_string: &str,
        ) -> ErrorIterator<'a> {
            if instance_string == "error" {
                error(ValidationError::unexpected(
                    instance,
                    "We're intentionally failing",
                ))
            } else {
                no_error()
            }
        }

        let mut config = CompilationConfig::default();
        config.add_content_media_type_check("test_media_type", Some(test_content_media_type_check));
        let compiled = JSONSchema::compile(schema, Some(config)).unwrap();
        compiled.is_valid(instance)
    }

    #[test_case(&json!({"contentMediaType": "application/json"}), &json!(false))]
    #[test_case(&json!({"contentMediaType": "application/json"}), &json!("{"))]
    fn test_custom_content_type_set_to_none_removes_the_handler(schema: &Value, instance: &Value) {
        let mut config = CompilationConfig::default();
        config.add_content_media_type_check("application/json", None);
        let compiled = JSONSchema::compile(schema, Some(config)).unwrap();
        assert!(compiled.is_valid(instance))
    }

    #[test_case(&json!({"contentEncoding": "base64"}), &json!("NDIK") => true)] // `echo "42" | base64` == "NDIK"
    #[test_case(&json!({"contentEncoding": "base64"}), &json!("a non-base64 string") => false)]
    #[test_case(&json!({"contentEncoding": "test_content_encoding"}), &json!("whatever") => true)]
    #[test_case(&json!({"contentEncoding": "test_content_encoding"}), &json!("error") => false)]
    fn test_custom_content_encoding(schema: &Value, instance: &Value) -> bool {
        fn test_content_encoding_check<'a>(
            instance: &'a Value,
            instance_string: &str,
        ) -> ErrorIterator<'a> {
            if instance_string == "error" {
                error(ValidationError::unexpected(
                    instance,
                    "We're intentionally failing",
                ))
            } else {
                no_error()
            }
        }

        let mut config = CompilationConfig::default();
        config
            .add_content_encoding_check("test_content_encoding", Some(test_content_encoding_check));
        let compiled = JSONSchema::compile(schema, Some(config)).unwrap();
        compiled.is_valid(instance)
    }

    #[test_case(&json!({"contentEncoding": "base64"}), &json!("NDIK"))] // `echo "42" | base64` == "NDIK"
    #[test_case(&json!({"contentEncoding": "base64"}), &json!("a non-base64 string"))]
    fn test_custom_content_encoding_set_to_none_removes_the_handler(
        schema: &Value,
        instance: &Value,
    ) {
        let mut config = CompilationConfig::default();
        config.add_content_encoding_check("base64", None);
        let compiled = JSONSchema::compile(schema, Some(config)).unwrap();
        assert!(compiled.is_valid(instance))
    }
}
