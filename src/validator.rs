use crate::{
    compilation::JSONSchema,
    error::{error, no_error, ErrorIterator, ValidationError},
};
use serde_json::{Map, Value};
use std::fmt;

pub(crate) trait Validate: Send + Sync + ToString {
    #[inline]
    fn build_validation_error<'a>(&self, instance: &'a Value) -> ValidationError<'a> {
        ValidationError::unexpected(instance, &self.to_string())
    }

    #[inline]
    fn is_valid_array(
        &self,
        _schema: &JSONSchema,
        _instance: &Value,
        _instance_value: &[Value],
    ) -> bool {
        true
    }
    #[inline]
    fn is_valid_boolean(
        &self,
        _schema: &JSONSchema,
        _instance: &Value,
        _instance_value: bool,
    ) -> bool {
        true
    }
    #[inline]
    fn is_valid_object(
        &self,
        _schema: &JSONSchema,
        _instance: &Value,
        _instance_value: &Map<String, Value>,
    ) -> bool {
        true
    }
    #[inline]
    fn is_valid_null(&self, _schema: &JSONSchema, _instance: &Value, _instance_value: ()) -> bool {
        true
    }
    #[inline]
    fn is_valid_number(
        &self,
        _schema: &JSONSchema,
        _instance: &Value,
        _instance_value: f64,
    ) -> bool {
        true
    }
    #[inline]
    fn is_valid_signed_integer(
        &self,
        _schema: &JSONSchema,
        _instance: &Value,
        _instance_value: i64,
    ) -> bool {
        true
    }
    #[inline]
    fn is_valid_string(
        &self,
        _schema: &JSONSchema,
        _instance: &Value,
        _instance_value: &str,
    ) -> bool {
        true
    }
    #[inline]
    fn is_valid_unsigned_integer(
        &self,
        _schema: &JSONSchema,
        _instance: &Value,
        _instance_value: u64,
    ) -> bool {
        true
    }
    #[inline]
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        match instance {
            Value::Array(instance_array) => self.is_valid_array(schema, instance, instance_array),
            Value::Bool(instance_boolean) => {
                self.is_valid_boolean(schema, instance, *instance_boolean)
            }
            Value::Null => self.is_valid_null(schema, instance, ()),
            Value::Number(instance_number) => {
                if let Some(instance_unsigned_integer) = instance_number.as_u64() {
                    self.is_valid_unsigned_integer(schema, instance, instance_unsigned_integer)
                } else if let Some(instance_signed_integer) = instance_number.as_i64() {
                    self.is_valid_signed_integer(schema, instance, instance_signed_integer)
                } else {
                    self.is_valid_number(
                        schema,
                        instance,
                        instance_number
                            .as_f64()
                            .expect("A JSON number will always be representable as f64"),
                    )
                }
            }
            Value::Object(instance_object) => {
                self.is_valid_object(schema, instance, instance_object)
            }
            Value::String(instance_string) => {
                self.is_valid_string(schema, instance, instance_string)
            }
        }
    }

    #[inline]
    fn validate_array<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_value: &'a [Value],
    ) -> ErrorIterator<'a> {
        if self.is_valid_array(schema, instance, instance_value) {
            no_error()
        } else {
            error(self.build_validation_error(instance))
        }
    }
    #[inline]
    fn validate_boolean<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_value: bool,
    ) -> ErrorIterator<'a> {
        if self.is_valid_boolean(schema, instance, instance_value) {
            no_error()
        } else {
            error(self.build_validation_error(instance))
        }
    }
    #[inline]
    fn validate_object<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_value: &'a Map<String, Value>,
    ) -> ErrorIterator<'a> {
        if self.is_valid_object(schema, instance, instance_value) {
            no_error()
        } else {
            error(self.build_validation_error(instance))
        }
    }
    #[inline]
    fn validate_null<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        _: (),
    ) -> ErrorIterator<'a> {
        if self.is_valid_null(schema, instance, ()) {
            no_error()
        } else {
            error(self.build_validation_error(instance))
        }
    }
    #[inline]
    fn validate_number<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_value: f64,
    ) -> ErrorIterator<'a> {
        if self.is_valid_number(schema, instance, instance_value) {
            no_error()
        } else {
            error(self.build_validation_error(instance))
        }
    }
    #[inline]
    fn validate_signed_integer<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_value: i64,
    ) -> ErrorIterator<'a> {
        if self.is_valid_signed_integer(schema, instance, instance_value) {
            no_error()
        } else {
            error(self.build_validation_error(instance))
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
            error(self.build_validation_error(instance))
        }
    }
    #[inline]
    fn validate_unsigned_integer<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_value: u64,
    ) -> ErrorIterator<'a> {
        if self.is_valid_unsigned_integer(schema, instance, instance_value) {
            no_error()
        } else {
            error(self.build_validation_error(instance))
        }
    }
    #[inline]
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        match instance {
            Value::Array(instance_array) => self.validate_array(schema, instance, instance_array),
            Value::Bool(instance_boolean) => {
                self.validate_boolean(schema, instance, *instance_boolean)
            }
            Value::Null => self.validate_null(schema, instance, ()),
            Value::Number(instance_number) => {
                if let Some(instance_unsigned_integer) = instance_number.as_u64() {
                    self.validate_unsigned_integer(schema, instance, instance_unsigned_integer)
                } else if let Some(instance_signed_integer) = instance_number.as_i64() {
                    self.validate_signed_integer(schema, instance, instance_signed_integer)
                } else {
                    self.validate_number(
                        schema,
                        instance,
                        instance_number
                            .as_f64()
                            .expect("A JSON number will always be representable as f64"),
                    )
                }
            }
            Value::Object(instance_object) => {
                self.validate_object(schema, instance, instance_object)
            }
            Value::String(instance_string) => {
                self.validate_string(schema, instance, instance_string)
            }
        }
    }
}

impl fmt::Debug for dyn Validate + Send + Sync {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_string())
    }
}
