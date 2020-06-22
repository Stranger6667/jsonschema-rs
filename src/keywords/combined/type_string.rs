//! Validators for `type` (`string` only), `const`, `maxLength` and `minLength` combined keywords.
use crate::{
    compilation::{CompilationContext, JSONSchema},
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::{basic as basic_validators, BoxedValidator},
    primitive_type::PrimitiveType,
    validator::Validate,
};
use serde_json::{Map, Value};

macro_rules! impl_is_valid_for_all_types_but_string {
    () => {
        #[inline]
        fn is_valid_array(&self, _: &JSONSchema, _: &Value, _: &[Value]) -> bool {
            false
        }
        #[inline]
        fn is_valid_boolean(&self, _: &JSONSchema, _: &Value, _: bool) -> bool {
            false
        }
        #[inline]
        fn is_valid_object(&self, _: &JSONSchema, _: &Value, _: &Map<String, Value>) -> bool {
            false
        }
        #[inline]
        fn is_valid_null(&self, _: &JSONSchema, _: &Value, _: ()) -> bool {
            false
        }
        #[inline]
        fn is_valid_number(&self, _: &JSONSchema, _: &Value, _: f64) -> bool {
            false
        }
        #[inline]
        fn is_valid_signed_integer(&self, _: &JSONSchema, _: &Value, _: i64) -> bool {
            false
        }
        #[inline]
        fn is_valid_unsigned_integer(&self, _: &JSONSchema, _: &Value, _: u64) -> bool {
            false
        }
    };
}

/// Combined validator for both `type` and `maxLength` keywords.
pub struct TypeStringMaxLength {
    max_length: u64,
}
impl TypeStringMaxLength {
    #[inline]
    pub(crate) fn compile(max_length: u64) -> BoxedValidator {
        Box::new(TypeStringMaxLength { max_length })
    }
}
impl Validate for TypeStringMaxLength {
    impl_is_valid_for_all_types_but_string!();
    #[inline]
    fn is_valid_string(&self, _: &JSONSchema, _: &Value, instance_value: &str) -> bool {
        instance_value.chars().count() as u64 <= self.max_length
    }
    #[inline]
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(instance_value) = instance {
            self.is_valid_string(schema, instance, instance_value)
        } else {
            false
        }
    }

    #[inline]
    fn validate_string<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_value: &'a str,
    ) -> ErrorIterator<'a> {
        if self.is_valid_string(schema, instance, instance_value) {
            no_error()
        } else {
            error(ValidationError::max_length(instance, self.max_length))
        }
    }

    #[inline]
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::String(instance_value) = instance {
            self.validate_string(schema, instance, instance_value)
        } else {
            error(ValidationError::single_type_error(
                instance,
                PrimitiveType::String,
            ))
        }
    }
}
impl ToString for TypeStringMaxLength {
    fn to_string(&self) -> String {
        format!("maxLength: {}, type: string", self.max_length)
    }
}

/// Combined validator for both `type` and `minLength` keywords.
pub struct TypeStringMinLength {
    min_length: u64,
}
impl TypeStringMinLength {
    #[inline]
    pub(crate) fn compile(min_length: u64) -> BoxedValidator {
        Box::new(TypeStringMinLength { min_length })
    }
}
impl Validate for TypeStringMinLength {
    impl_is_valid_for_all_types_but_string!();
    #[inline]
    fn is_valid_string(&self, _: &JSONSchema, _: &Value, instance_value: &str) -> bool {
        instance_value.chars().count() as u64 >= self.min_length
    }
    #[inline]
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(instance_value) = instance {
            self.is_valid_string(schema, instance, instance_value)
        } else {
            false
        }
    }

    #[inline]
    fn validate_string<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_value: &'a str,
    ) -> ErrorIterator<'a> {
        if self.is_valid_string(schema, instance, instance_value) {
            no_error()
        } else {
            error(ValidationError::min_length(instance, self.min_length))
        }
    }

    #[inline]
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::String(instance_value) = instance {
            self.validate_string(schema, instance, instance_value)
        } else {
            error(ValidationError::single_type_error(
                instance,
                PrimitiveType::String,
            ))
        }
    }
}
impl ToString for TypeStringMinLength {
    fn to_string(&self) -> String {
        format!("minLength: {}, type: string", self.min_length)
    }
}
/// Combined validator for both `type`, `maxLength` and `minLength` keywords.
pub struct TypeStringMaxLengthMinLength {
    max_length: u64,
    min_length: u64,
}
impl TypeStringMaxLengthMinLength {
    #[inline]
    pub(crate) fn compile(max_length: u64, min_length: u64) -> BoxedValidator {
        Box::new(TypeStringMaxLengthMinLength {
            max_length,
            min_length,
        })
    }
}
impl Validate for TypeStringMaxLengthMinLength {
    impl_is_valid_for_all_types_but_string!();
    #[inline]
    fn is_valid_string(&self, _: &JSONSchema, _: &Value, instance_value: &str) -> bool {
        let char_count = instance_value.chars().count() as u64;
        char_count >= self.min_length && char_count <= self.max_length
    }
    #[inline]
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(instance_value) = instance {
            self.is_valid_string(schema, instance, instance_value)
        } else {
            false
        }
    }

    #[inline]
    fn validate_string<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_value: &'a str,
    ) -> ErrorIterator<'a> {
        if self.is_valid_string(schema, instance, instance_value) {
            no_error()
        } else if instance_value.chars().count() as u64 <= self.min_length {
            error(ValidationError::min_length(instance, self.min_length))
        } else {
            error(ValidationError::max_length(instance, self.max_length))
        }
    }

    #[inline]
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::String(instance_value) = instance {
            self.validate_string(schema, instance, instance_value)
        } else {
            error(ValidationError::single_type_error(
                instance,
                PrimitiveType::String,
            ))
        }
    }
}

impl ToString for TypeStringMaxLengthMinLength {
    fn to_string(&self) -> String {
        format!(
            "maxLength: {}, minLength: {}, type: string",
            self.max_length, self.min_length
        )
    }
}

#[inline]
pub fn compile(schema: &Map<String, Value>, _: &CompilationContext) -> Option<BoxedValidator> {
    if Some("string") == schema.get("type").and_then(Value::as_str) {
        Some(
            match (
                schema.get("const"),
                schema.get("maxLength").and_then(Value::as_u64),
                schema.get("minLength").and_then(Value::as_u64),
            ) {
                (Some(Value::String(const_string)), max_length, min_length) => {
                    let const_string_length = const_string.chars().count() as u64;
                    if min_length.unwrap_or(0) <= const_string_length
                        && const_string_length <= max_length.unwrap_or(u64::MAX)
                    {
                        // The constant is a string and within the range defined by maximum and minimum
                        basic_validators::const_::ConstStringValidator::compile(const_string)
                            .expect("We're intentionally building it, we know that it is correct")
                    } else {
                        // The constant is a string but either longer than maxLength or shorter than minLength
                        basic_validators::boolean::FalseValidator::compile()
                            .expect("We're intentionally building it, we know that it is correct")
                    }
                }
                (Some(_), _, _) => {
                    // The constant is not a string but type is string
                    basic_validators::boolean::FalseValidator::compile()
                        .expect("We're intentionally building it, we know that it is correct")
                }
                (None, Some(max_length), Some(min_length)) => {
                    TypeStringMaxLengthMinLength::compile(max_length, min_length)
                }
                (None, Some(max_length), None) => TypeStringMaxLength::compile(max_length),
                (None, None, Some(min_length)) => TypeStringMinLength::compile(min_length),
                _ => return None,
            },
        )
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::compile;
    use crate::compilation::{CompilationContext, JSONSchema};
    use serde_json::{json, Value};
    use test_case::test_case;

    // Missing expected keys
    #[test_case(&json!({}))]
    #[test_case(&json!({"type": "string"}))]
    #[test_case(&json!({"maxLength": 2}))]
    #[test_case(&json!({"minLength": 1}))]
    #[test_case(&json!({"maxLength": 2, "minLength": 1}))]
    // Invalid keys
    #[test_case(&json!({"type": "string", "maxLength": "1"}))]
    #[test_case(&json!({"type": "string", "minLength": "1"}))]
    #[test_case(&json!({"type": "string", "maxLength": "1", "minLength": "1"}))]
    fn test_no_validator_is_built(schema: &Value) {
        assert!(compile(&schema.as_object().unwrap(), &CompilationContext::default()).is_none())
    }

    #[test_case(&json!({"type": "string", "const": "string"}))]
    #[test_case(&json!({"type": "string", "maxLength": 1}))]
    #[test_case(&json!({"type": "string", "minLength": 1}))]
    #[test_case(&json!({"type": "string", "maxLength": 1, "minLength": 1}))]
    fn test_validator_is_built(schema: &Value) {
        assert!(compile(&schema.as_object().unwrap(), &CompilationContext::default()).is_some())
    }

    #[test_case(&json!({"type": "string", "const": "string"}), &json!("1") => false)]
    #[test_case(&json!({"type": "string", "const": "string"}), &json!("string") => true)]
    #[test_case(&json!({"type": "string", "const": 1}), &json!("string") => false)]
    #[test_case(&json!({"type": "string", "const": "string", "maxLength": 3}), &json!("012") => false)]
    #[test_case(&json!({"type": "string", "const": "string", "minLength": 8}), &json!("this-is-very-long") => false)]
    #[test_case(&json!({"type": "string", "maxLength": 3}), &json!(1) => false)]
    #[test_case(&json!({"type": "string", "maxLength": 3}), &json!("012") => true)]
    #[test_case(&json!({"type": "string", "maxLength": 3}), &json!("0123") => false)]
    #[test_case(&json!({"type": "string", "minLength": 1}), &json!(1) => false)]
    #[test_case(&json!({"type": "string", "minLength": 1}), &json!("") => false)]
    #[test_case(&json!({"type": "string", "minLength": 1}), &json!("0") => true)]
    #[test_case(&json!({"type": "string", "maxLength": 3, "minLength": 1}), &json!(1) => false)]
    #[test_case(&json!({"type": "string", "maxLength": 3, "minLength": 1}), &json!("") => false)]
    #[test_case(&json!({"type": "string", "maxLength": 3, "minLength": 1}), &json!("0") => true)]
    #[test_case(&json!({"type": "string", "maxLength": 3, "minLength": 1}), &json!("012") => true)]
    #[test_case(&json!({"type": "string", "maxLength": 3, "minLength": 1}), &json!("0123") => false)]
    fn test_is_valid(schema: &Value, instance: &Value) -> bool {
        let jsonschema = JSONSchema::compile(schema, None).unwrap();
        let validator =
            compile(&schema.as_object().unwrap(), &CompilationContext::default()).unwrap();
        validator.is_valid(&jsonschema, instance)
    }
}
