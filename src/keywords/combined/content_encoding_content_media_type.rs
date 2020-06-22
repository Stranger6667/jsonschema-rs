//! Validators for `contentEncoding` and `contentMediaType` combined keywords.
use crate::{
    compilation::{CompilationContext, JSONSchema},
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::{basic::content_media_type::is_json, BoxedValidator},
    validator::Validate,
};
use serde_json::{Map, Value};

pub fn from_base64<'a>(
    instance: &'a Value,
    instance_string: &str,
) -> Result<String, ValidationError<'a>> {
    match base64::decode(instance_string) {
        Ok(value) => Ok(String::from_utf8(value)?),
        Err(_) => Err(ValidationError::format(instance, "base64")),
    }
}

/// Combined validator for both `contentEncoding` and `contentMediaType` keywords.
pub struct ContentEncodingAndContentMediaTypeValidator {
    media_type: String,
    encoding: String,
    func: for<'a> fn(&'a Value, &str) -> ErrorIterator<'a>,
    converter: for<'a> fn(&'a Value, &str) -> Result<String, ValidationError<'a>>,
}
impl ContentEncodingAndContentMediaTypeValidator {
    #[inline]
    pub(crate) fn compile(
        media_type: &str,
        encoding: &str,
        func: for<'a> fn(&'a Value, &str) -> ErrorIterator<'a>,
        converter: for<'a> fn(&'a Value, &str) -> Result<String, ValidationError<'a>>,
    ) -> BoxedValidator {
        Box::new(ContentEncodingAndContentMediaTypeValidator {
            media_type: media_type.to_string(),
            encoding: encoding.to_string(),
            func,
            converter,
        })
    }
}
/// Decode the input value & check media type
impl Validate for ContentEncodingAndContentMediaTypeValidator {
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

#[inline]
pub fn compile(schema: &Map<String, Value>, _: &CompilationContext) -> Option<BoxedValidator> {
    let (content_encoding, converter) = match schema.get("contentEncoding").and_then(Value::as_str)
    {
        Some("base64") => ("base64", from_base64),
        _ => return None,
    };

    let (content_media_type, func) = match schema.get("contentMediaType").and_then(Value::as_str) {
        Some("application/json") => ("application/json", is_json),
        _ => return None,
    };

    Some(ContentEncodingAndContentMediaTypeValidator::compile(
        content_media_type,
        content_encoding,
        func,
        converter,
    ))
}

#[cfg(test)]
mod tests {
    use super::compile;
    use crate::compilation::{CompilationContext, JSONSchema};
    use base64::encode;
    use serde_json::{json, Value};
    use test_case::test_case;

    // Missing expected keys
    #[test_case(&json!({}))]
    #[test_case(&json!({"contentEncoding": "base64"}))]
    #[test_case(&json!({"contentMediaType": "application/json"}))]
    // Invalid keys
    #[test_case(&json!({"contentEncoding": 1}))]
    #[test_case(&json!({"contentMediaType": 1}))]
    fn test_no_validator_is_built(schema: &Value) {
        assert!(compile(&schema.as_object().unwrap(), &CompilationContext::default()).is_none())
    }

    #[test_case(&json!({"contentEncoding": "base64", "contentMediaType": "application/json"}))]
    fn test_validator_is_built(schema: &Value) {
        assert!(compile(&schema.as_object().unwrap(), &CompilationContext::default()).is_some())
    }

    #[test_case(&json!({"contentEncoding": "base64", "contentMediaType": "application/json"}), &json!(1) => true)]
    #[test_case(&json!({"contentEncoding": "base64", "contentMediaType": "application/json"}), &json!("something-that-is-no-base64") => false)]
    #[test_case(&json!({"contentEncoding": "base64", "contentMediaType": "application/json"}), &json!(encode("something-that-is-no-json")) => false)]
    #[test_case(&json!({"contentEncoding": "base64", "contentMediaType": "application/json"}), &json!(encode("{}")) => true)]
    fn test_is_valid(schema: &Value, instance: &Value) -> bool {
        let jsonschema = JSONSchema::compile(schema, None).unwrap();
        let validator =
            compile(&schema.as_object().unwrap(), &CompilationContext::default()).unwrap();
        validator.is_valid(&jsonschema, instance)
    }
}
