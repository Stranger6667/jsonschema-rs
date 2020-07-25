use crate::{
    compilation::{context::CompilationContext, JSONSchema},
    error::{CompilationError, ValidationError},
    keywords::CompilationResult,
    validator::Validate,
};
use serde_json::{Map, Value};
use std::f64::EPSILON;

#[derive(Debug)]
pub(crate) struct EnumValidator {
    options: Value,
    items: Vec<Value>,
}

impl EnumValidator {
    #[inline]
    pub(crate) fn compile(schema: &Value) -> CompilationResult {
        if let Value::Array(items) = schema {
            Ok(Box::new(EnumValidator {
                options: schema.clone(),
                items: items.clone(),
            }))
        } else {
            Err(CompilationError::SchemaError)
        }
    }
}

impl Validate for EnumValidator {
    #[inline]
    fn build_validation_error<'a>(&self, instance: &'a Value) -> ValidationError<'a> {
        ValidationError::enumeration(instance, &self.options)
    }

    #[inline]
    fn is_valid_array(&self, _: &JSONSchema, _: &Value, instance_value: &[Value]) -> bool {
        self.items.iter().any(|item| {
            if let Value::Array(value) = item {
                value.as_slice() == instance_value
            } else {
                false
            }
        })
    }
    #[inline]
    fn is_valid_boolean(&self, _: &JSONSchema, _: &Value, instance_value: bool) -> bool {
        self.items.iter().any(|item| {
            if let Value::Bool(value) = item {
                *value == instance_value
            } else {
                false
            }
        })
    }
    #[inline]
    fn is_valid_object(
        &self,
        _: &JSONSchema,
        _: &Value,
        instance_value: &Map<String, Value>,
    ) -> bool {
        self.items.iter().any(|item| {
            if let Value::Object(value) = item {
                value == instance_value
            } else {
                false
            }
        })
    }
    #[inline]
    fn is_valid_null(&self, _: &JSONSchema, _: &Value, _: ()) -> bool {
        self.items.iter().any(Value::is_null)
    }
    #[inline]
    fn is_valid_number(&self, _: &JSONSchema, _: &Value, instance_value: f64) -> bool {
        self.items.iter().any(|item| {
            item.as_f64()
                .map_or_else(|| false, |value| (value - instance_value).abs() < EPSILON)
        })
    }
    #[inline]
    fn is_valid_signed_integer(&self, _: &JSONSchema, _: &Value, instance_value: i64) -> bool {
        self.items.iter().any(|item| {
            item.as_i64()
                .map_or_else(|| false, |value| value == instance_value)
        })
    }
    #[inline]
    fn is_valid_string(&self, _: &JSONSchema, _: &Value, instance_value: &str) -> bool {
        self.items.iter().any(|item| {
            if let Value::String(value) = item {
                value == instance_value
            } else {
                false
            }
        })
    }
    #[inline]
    fn is_valid_unsigned_integer(&self, _: &JSONSchema, _: &Value, instance_value: u64) -> bool {
        self.items.iter().any(|item| {
            item.as_u64()
                .map_or_else(|| false, |value| value == instance_value)
        })
    }
}
impl ToString for EnumValidator {
    fn to_string(&self) -> String {
        format!(
            "enum: [{}]",
            self.items
                .iter()
                .map(Value::to_string)
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

#[inline]
pub(crate) fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    Some(EnumValidator::compile(schema))
}
