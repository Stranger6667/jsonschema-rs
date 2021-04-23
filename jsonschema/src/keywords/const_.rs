use crate::{
    compilation::{context::CompilationContext, JSONSchema},
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::{helpers, CompilationResult},
    validator::Validate,
};
use serde_json::{Map, Number, Value};
use std::f64::EPSILON;

use super::InstancePath;

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
    fn validate<'a, 'b>(
        &'b self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        curr_instance_path: InstancePath<'b>,
    ) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::constant_array(
                curr_instance_path.into(),
                instance,
                &self.value,
            ))
        }
    }

    #[inline]
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::Array(instance_value) = instance {
            helpers::equal_arrays(&self.value, instance_value)
        } else {
            false
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
    fn validate<'a, 'b>(
        &'b self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        curr_instance_path: InstancePath<'b>,
    ) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::constant_boolean(
                curr_instance_path.into(),
                instance,
                self.value,
            ))
        }
    }

    #[inline]
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::Bool(instance_value) = instance {
            &self.value == instance_value
        } else {
            false
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
    fn validate<'a, 'b>(
        &'b self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        curr_instance_path: InstancePath<'b>,
    ) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::constant_null(
                curr_instance_path.into(),
                instance,
            ))
        }
    }

    #[inline]
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        instance.is_null()
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
    fn validate<'a, 'b>(
        &'b self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        curr_instance_path: InstancePath<'b>,
    ) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::constant_number(
                curr_instance_path.into(),
                instance,
                &self.original_value,
            ))
        }
    }

    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::Number(item) = instance {
            (self.value - item.as_f64().expect("Always representable as f64")).abs() < EPSILON
        } else {
            false
        }
    }
}

impl ToString for ConstNumberValidator {
    fn to_string(&self) -> String {
        format!("const: {}", self.original_value)
    }
}

pub(crate) struct ConstObjectValidator {
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
    fn validate<'a, 'b>(
        &'b self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        curr_instance_path: InstancePath<'b>,
    ) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::constant_object(
                curr_instance_path.into(),
                instance,
                &self.value,
            ))
        }
    }

    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            helpers::equal_objects(&self.value, item)
        } else {
            false
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

pub(crate) struct ConstStringValidator {
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
    fn validate<'a, 'b>(
        &'b self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        curr_instance_path: InstancePath<'b>,
    ) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::constant_string(
                curr_instance_path.into(),
                instance,
                &self.value,
            ))
        }
    }

    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            &self.value == item
        } else {
            false
        }
    }
}

impl ToString for ConstStringValidator {
    fn to_string(&self) -> String {
        format!("const: {}", self.value)
    }
}

#[inline]
pub(crate) fn compile(
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
