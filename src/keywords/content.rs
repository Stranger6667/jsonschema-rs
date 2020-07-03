//! Validators for `contentMediaType` and `contentEncoding` keywords.
use crate::{
    compilation::{context::CompilationContext, JSONSchema},
    content_encoding::{ContentEncodingCheckType, ContentEncodingConverterType},
    content_media_type::ContentMediaTypeCheckType,
    error::{error, no_error, CompilationError, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    validator::Validate,
};
use serde_json::{Map, Value};

/// Validator for `contentMediaType` keyword.
pub(crate) struct ContentMediaTypeValidator {
    media_type: String,
    func: ContentMediaTypeCheckType,
}

impl ContentMediaTypeValidator {
    #[inline]
    pub(crate) fn compile(media_type: &str, func: ContentMediaTypeCheckType) -> CompilationResult {
        Ok(Box::new(ContentMediaTypeValidator {
            media_type: media_type.to_string(),
            func,
        }))
    }
}

/// Validator delegates validation to the stored function.
impl Validate for ContentMediaTypeValidator {
    #[inline]
    fn is_valid_string(&self, _: &JSONSchema, _: &Value, instance_value: &str) -> bool {
        (self.func)(instance_value)
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
        if (self.func)(instance_value) {
            no_error()
        } else {
            error(ValidationError::content_media_type(
                instance,
                &self.media_type,
            ))
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
impl ToString for ContentMediaTypeValidator {
    fn to_string(&self) -> String {
        format!("contentMediaType: {}", self.media_type)
    }
}

/// Validator for `contentEncoding` keyword.
pub(crate) struct ContentEncodingValidator {
    encoding: String,
    func: ContentEncodingCheckType,
}

impl ContentEncodingValidator {
    #[inline]
    pub(crate) fn compile(encoding: &str, func: ContentEncodingCheckType) -> CompilationResult {
        Ok(Box::new(ContentEncodingValidator {
            encoding: encoding.to_string(),
            func,
        }))
    }
}

impl Validate for ContentEncodingValidator {
    #[inline]
    fn is_valid_string(&self, _: &JSONSchema, _: &Value, instance_value: &str) -> bool {
        (self.func)(instance_value)
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
        if (self.func)(instance_value) {
            no_error()
        } else {
            error(ValidationError::content_encoding(instance, &self.encoding))
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
}

impl ContentMediaTypeAndEncodingValidator {
    #[inline]
    pub(crate) fn compile(
        media_type: &str,
        encoding: &str,
        func: ContentMediaTypeCheckType,
        converter: ContentEncodingConverterType,
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
    fn is_valid_string(&self, _: &JSONSchema, _: &Value, instance_value: &str) -> bool {
        match (self.converter)(instance_value) {
            Ok(None) | Err(_) => false,
            Ok(Some(converted)) => (self.func)(&converted),
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
        match (self.converter)(instance_value) {
            Ok(None) => error(ValidationError::content_encoding(instance, &self.encoding)),
            Ok(Some(converted)) => {
                if (self.func)(&converted) {
                    no_error()
                } else {
                    error(ValidationError::content_media_type(
                        instance,
                        &self.media_type,
                    ))
                }
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
pub(crate) fn compile_media_type(
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
pub(crate) fn compile_content_encoding(
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
    use crate::{error::ValidationError, JSONSchema};
    use serde_json::{json, Value};
    use test_case::test_case;
    fn converter_custom_encoding(
        instance_string: &str,
    ) -> Result<Option<String>, ValidationError<'static>> {
        if let Some(first_space_index) = instance_string.find(' ') {
            if let Ok(value) = instance_string[..first_space_index].parse::<u64>() {
                if instance_string[first_space_index..].chars().count() == value as usize {
                    return Ok(Some(instance_string[first_space_index..].to_string()));
                }
            }
        }
        Ok(None)
    }
    fn check_custom_encoding(instance_string: &str) -> bool {
        if let Some(first_space_index) = instance_string.find(' ') {
            if let Ok(value) = instance_string[..first_space_index].parse::<u64>() {
                return instance_string[(first_space_index + 1)..].chars().count()
                    == value as usize;
            }
        }
        false
    }

    #[test_case(&json!({"contentMediaType": "application/json"}), &json!("{}") => true)]
    #[test_case(&json!({"contentMediaType": "application/json"}), &json!("{") => false)]
    #[test_case(&json!({"contentMediaType": "test_media_type"}), &json!("whatever") => true)]
    #[test_case(&json!({"contentMediaType": "test_media_type"}), &json!("error") => false)]
    fn test_with_content_media_type(schema: &Value, instance: &Value) -> bool {
        fn test_content_media_type_check(instance_string: &str) -> bool {
            instance_string != "error"
        }

        let compiled = JSONSchema::options()
            .with_content_media_type("test_media_type", test_content_media_type_check)
            .compile(schema)
            .unwrap();
        compiled.is_valid(instance)
    }

    #[test_case(&json!({"contentMediaType": "application/json"}), &json!(false))]
    #[test_case(&json!({"contentMediaType": "application/json"}), &json!("{"))]
    fn test_without_content_media_type_support(schema: &Value, instance: &Value) {
        let compiled = JSONSchema::options()
            .without_content_media_type_support("application/json")
            .compile(schema)
            .unwrap();
        assert!(compiled.is_valid(instance))
    }

    #[test_case(&json!({"contentEncoding": "base64"}), &json!("NDIK") => true)] // `echo "42" | base64` == "NDIK"
    #[test_case(&json!({"contentEncoding": "base64"}), &json!("a non-base64 string") => false)]
    #[test_case(&json!({"contentEncoding": "test_content_encoding"}), &json!("whatever") => false)]
    #[test_case(&json!({"contentEncoding": "test_content_encoding"}), &json!("1 a") => true)]
    #[test_case(&json!({"contentEncoding": "test_content_encoding"}), &json!("3 some") => false)]
    fn test_custom_content_encoding(schema: &Value, instance: &Value) -> bool {
        let compiled = JSONSchema::options()
            .with_content_encoding(
                "test_content_encoding",
                check_custom_encoding,
                converter_custom_encoding,
            )
            .compile(schema)
            .unwrap();
        compiled.is_valid(instance)
    }

    #[test_case(&json!({"contentEncoding": "base64"}), &json!("NDIK"))] // `echo "42" | base64` == "NDIK"
    #[test_case(&json!({"contentEncoding": "base64"}), &json!("a non-base64 string"))]
    fn test_custom_content_encoding_set_to_none_removes_the_handler(
        schema: &Value,
        instance: &Value,
    ) {
        let compiled = JSONSchema::options()
            .without_content_encoding_support("base64")
            .compile(schema)
            .unwrap();
        assert!(compiled.is_valid(instance))
    }

    #[test_case("2 {" => false)] // Content Encoding not respected
    #[test_case("2 {a" => false)] // Content Media Type not respected
    #[test_case("2 {}" => true)]
    fn test_custom_media_type_and_encoding(instance: &Value) -> bool {
        let schema = json!({
            "contentMediaType": "application/json",
            "contentEncoding": "prefix-string"
        });
        let compiled = JSONSchema::options()
            .with_content_encoding(
                "prefix-string",
                check_custom_encoding,
                converter_custom_encoding,
            )
            .compile(&schema)
            .unwrap();
        compiled.is_valid(instance)
    }
}
