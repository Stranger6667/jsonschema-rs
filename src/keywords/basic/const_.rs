use crate::{
    compilation::{CompilationContext, JSONSchema},
    error::{error, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    validator::Validate,
};
use serde_json::{Map, Number, Value};
use std::f64::EPSILON;

struct ConstArrayValidator {
    value: Vec<Value>,
}
impl ConstArrayValidator {
    #[inline]
    pub(crate) fn compile(value: &[Value]) -> CompilationResult {
        Ok(Box::new(ConstArrayValidator {
            value: value.to_vec(),
        }))
    }
}
impl Validate for ConstArrayValidator {
    #[inline]
    fn build_validation_error<'a>(&self, instance: &'a Value) -> ValidationError<'a> {
        ValidationError::constant_array(instance, &self.value)
    }

    #[inline]
    fn is_valid_array(&self, _: &JSONSchema, _: &Value, instance_value: &[Value]) -> bool {
        self.value == instance_value
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
    fn is_valid_string(&self, _: &JSONSchema, _: &Value, _: &str) -> bool {
        false
    }
    #[inline]
    fn is_valid_unsigned_integer(&self, _: &JSONSchema, _: &Value, _: u64) -> bool {
        false
    }
    #[inline]
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Array(instance_value) = instance {
            self.is_valid_array(schema, instance, instance_value)
        } else {
            false
        }
    }

    #[inline]
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Array(instance_value) = instance {
            self.validate_array(schema, instance, instance_value)
        } else {
            error(self.build_validation_error(instance))
        }
    }
}
impl ToString for ConstArrayValidator {
    fn to_string(&self) -> String {
        format!(
            "const: [{}]",
            self.value
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

struct ConstBooleanValidator {
    value: bool,
}
impl ConstBooleanValidator {
    #[inline]
    pub(crate) fn compile(value: bool) -> CompilationResult {
        Ok(Box::new(ConstBooleanValidator { value }))
    }
}
impl Validate for ConstBooleanValidator {
    #[inline]
    fn build_validation_error<'a>(&self, instance: &'a Value) -> ValidationError<'a> {
        ValidationError::constant_boolean(instance, self.value)
    }

    #[inline]
    fn is_valid_array(&self, _: &JSONSchema, _: &Value, _: &[Value]) -> bool {
        false
    }
    #[inline]
    fn is_valid_boolean(&self, _: &JSONSchema, _: &Value, instance_value: bool) -> bool {
        self.value == instance_value
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
    fn is_valid_object(&self, _: &JSONSchema, _: &Value, _: &Map<String, Value>) -> bool {
        false
    }
    #[inline]
    fn is_valid_signed_integer(&self, _: &JSONSchema, _: &Value, _: i64) -> bool {
        false
    }
    #[inline]
    fn is_valid_string(&self, _: &JSONSchema, _: &Value, _: &str) -> bool {
        false
    }
    #[inline]
    fn is_valid_unsigned_integer(&self, _: &JSONSchema, _: &Value, _: u64) -> bool {
        false
    }
    #[inline]
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Bool(instance_value) = instance {
            self.is_valid_boolean(schema, instance, *instance_value)
        } else {
            false
        }
    }

    #[inline]
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Bool(instance_value) = instance {
            self.validate_boolean(schema, instance, *instance_value)
        } else {
            error(self.build_validation_error(instance))
        }
    }
}
impl ToString for ConstBooleanValidator {
    fn to_string(&self) -> String {
        format!("const: {}", self.value)
    }
}

struct ConstNullValidator {}
impl ConstNullValidator {
    #[inline]
    pub(crate) fn compile() -> CompilationResult {
        Ok(Box::new(ConstNullValidator {}))
    }
}
impl Validate for ConstNullValidator {
    #[inline]
    fn build_validation_error<'a>(&self, instance: &'a Value) -> ValidationError<'a> {
        ValidationError::constant_null(instance)
    }

    #[inline]
    fn is_valid_array(&self, _: &JSONSchema, _: &Value, _: &[Value]) -> bool {
        false
    }
    #[inline]
    fn is_valid_boolean(&self, _: &JSONSchema, _: &Value, _: bool) -> bool {
        false
    }
    #[inline]
    fn is_valid_null(&self, _: &JSONSchema, _: &Value, _: ()) -> bool {
        true
    }
    #[inline]
    fn is_valid_number(&self, _: &JSONSchema, _: &Value, _: f64) -> bool {
        false
    }
    #[inline]
    fn is_valid_object(&self, _: &JSONSchema, _: &Value, _: &Map<String, Value>) -> bool {
        false
    }
    #[inline]
    fn is_valid_signed_integer(&self, _: &JSONSchema, _: &Value, _: i64) -> bool {
        false
    }
    #[inline]
    fn is_valid_string(&self, _: &JSONSchema, _: &Value, _: &str) -> bool {
        false
    }
    #[inline]
    fn is_valid_unsigned_integer(&self, _: &JSONSchema, _: &Value, _: u64) -> bool {
        false
    }
    #[inline]
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Null = instance {
            self.is_valid_null(schema, instance, ())
        } else {
            false
        }
    }

    #[inline]
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Null = instance {
            self.validate_null(schema, instance, ())
        } else {
            error(self.build_validation_error(instance))
        }
    }
}
impl ToString for ConstNullValidator {
    fn to_string(&self) -> String {
        format!("const: {}", Value::Null)
    }
}

struct ConstNumberValidator {
    // This is saved in order to ensure that the error message is not altered by precision loss
    original_value: Number,
    value: f64,
}
impl ConstNumberValidator {
    #[inline]
    pub(crate) fn compile(original_value: &Number) -> CompilationResult {
        Ok(Box::new(ConstNumberValidator {
            original_value: original_value.clone(),
            value: original_value
                .as_f64()
                .expect("A JSON number will always be representable as f64"),
        }))
    }
}
impl Validate for ConstNumberValidator {
    #[inline]
    fn build_validation_error<'a>(&self, instance: &'a Value) -> ValidationError<'a> {
        ValidationError::constant_number(instance, &self.original_value)
    }

    #[inline]
    fn is_valid_array(&self, _: &JSONSchema, _: &Value, _: &[Value]) -> bool {
        false
    }
    #[inline]
    fn is_valid_boolean(&self, _: &JSONSchema, _: &Value, _: bool) -> bool {
        false
    }
    #[inline]
    fn is_valid_null(&self, _: &JSONSchema, _: &Value, _: ()) -> bool {
        false
    }
    #[inline]
    fn is_valid_number(&self, _: &JSONSchema, _: &Value, instance_value: f64) -> bool {
        (self.value - instance_value).abs() < EPSILON
    }
    #[inline]
    fn is_valid_object(&self, _: &JSONSchema, _: &Value, _: &Map<String, Value>) -> bool {
        false
    }
    #[inline]
    fn is_valid_signed_integer(
        &self,
        schema: &JSONSchema,
        instance: &Value,
        instance_value: i64,
    ) -> bool {
        #[allow(clippy::cast_precision_loss)]
        self.is_valid_number(schema, instance, instance_value as f64)
    }
    #[inline]
    fn is_valid_string(&self, _: &JSONSchema, _: &Value, _: &str) -> bool {
        false
    }
    #[inline]
    fn is_valid_unsigned_integer(
        &self,
        schema: &JSONSchema,
        instance: &Value,
        instance_value: u64,
    ) -> bool {
        #[allow(clippy::cast_precision_loss)]
        self.is_valid_number(schema, instance, instance_value as f64)
    }
    #[inline]
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Some(instance_value) = instance.as_f64() {
            self.is_valid_number(schema, instance, instance_value)
        } else {
            false
        }
    }

    #[inline]
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Some(instance_value) = instance.as_f64() {
            self.validate_number(schema, instance, instance_value)
        } else {
            error(self.build_validation_error(instance))
        }
    }
}
impl ToString for ConstNumberValidator {
    fn to_string(&self) -> String {
        format!("const: {}", self.original_value)
    }
}

struct ConstObjectValidator {
    value: Map<String, Value>,
}
impl ConstObjectValidator {
    #[inline]
    pub(crate) fn compile(value: &Map<String, Value>) -> CompilationResult {
        Ok(Box::new(ConstObjectValidator {
            value: value.clone(),
        }))
    }
}
impl Validate for ConstObjectValidator {
    #[inline]
    fn build_validation_error<'a>(&self, instance: &'a Value) -> ValidationError<'a> {
        ValidationError::constant_object(instance, &self.value)
    }

    #[inline]
    fn is_valid_array(&self, _: &JSONSchema, _: &Value, _: &[Value]) -> bool {
        false
    }
    #[inline]
    fn is_valid_boolean(&self, _: &JSONSchema, _: &Value, _: bool) -> bool {
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
    fn is_valid_object(
        &self,
        _: &JSONSchema,
        _: &Value,
        instance_value: &Map<String, Value>,
    ) -> bool {
        &self.value == instance_value
    }
    #[inline]
    fn is_valid_signed_integer(&self, _: &JSONSchema, _: &Value, _: i64) -> bool {
        false
    }
    #[inline]
    fn is_valid_string(&self, _: &JSONSchema, _: &Value, _: &str) -> bool {
        false
    }
    #[inline]
    fn is_valid_unsigned_integer(&self, _: &JSONSchema, _: &Value, _: u64) -> bool {
        false
    }
    #[inline]
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(instance_value) = instance {
            self.is_valid_object(schema, instance, instance_value)
        } else {
            false
        }
    }

    #[inline]
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Object(instance_value) = instance {
            self.validate_object(schema, instance, instance_value)
        } else {
            error(self.build_validation_error(instance))
        }
    }
}
impl ToString for ConstObjectValidator {
    fn to_string(&self) -> String {
        format!(
            "const: {{{}}}",
            self.value
                .iter()
                .map(|(key, value)| format!(r#""{}":{}"#, key, value))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

struct ConstStringValidator {
    value: String,
}
impl ConstStringValidator {
    #[inline]
    pub(crate) fn compile(value: &str) -> CompilationResult {
        Ok(Box::new(ConstStringValidator {
            value: value.to_string(),
        }))
    }
}
impl Validate for ConstStringValidator {
    #[inline]
    fn build_validation_error<'a>(&self, instance: &'a Value) -> ValidationError<'a> {
        ValidationError::constant_string(instance, &self.value)
    }

    #[inline]
    fn is_valid_array(&self, _: &JSONSchema, _: &Value, _: &[Value]) -> bool {
        false
    }
    #[inline]
    fn is_valid_boolean(&self, _: &JSONSchema, _: &Value, _: bool) -> bool {
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
    fn is_valid_object(&self, _: &JSONSchema, _: &Value, _: &Map<String, Value>) -> bool {
        false
    }
    #[inline]
    fn is_valid_signed_integer(&self, _: &JSONSchema, _: &Value, _: i64) -> bool {
        false
    }
    #[inline]
    fn is_valid_string(&self, _: &JSONSchema, _: &Value, instance_value: &str) -> bool {
        self.value == instance_value
    }
    #[inline]
    fn is_valid_unsigned_integer(&self, _: &JSONSchema, _: &Value, _: u64) -> bool {
        false
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
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::String(instance_value) = instance {
            self.validate_string(schema, instance, instance_value)
        } else {
            error(self.build_validation_error(instance))
        }
    }
}
impl ToString for ConstStringValidator {
    fn to_string(&self) -> String {
        format!(r#"const: "{}""#, self.value)
    }
}

#[inline]
pub fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    match schema {
        Value::Array(items) => Some(ConstArrayValidator::compile(items)),
        Value::Bool(item) => Some(ConstBooleanValidator::compile(*item)),
        Value::Null => Some(ConstNullValidator::compile()),
        Value::Number(item) => Some(ConstNumberValidator::compile(item)),
        Value::Object(map) => Some(ConstObjectValidator::compile(map)),
        Value::String(string) => Some(ConstStringValidator::compile(string)),
    }
}
