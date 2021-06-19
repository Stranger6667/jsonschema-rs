use crate::{
    compilation::{context::CompilationContext, JSONSchema},
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::{helpers, CompilationResult},
    validator::Validate,
};
use serde_json::{Map, Number, Value};
use std::f64::EPSILON;

use crate::paths::{InstancePath, JSONPointer};

struct ConstArrayValidator {
    value: Vec<Value>,
    schema_path: JSONPointer,
}
impl ConstArrayValidator {
    #[inline]
    pub(crate) fn compile(value: &[Value], schema_path: JSONPointer) -> CompilationResult {
        Ok(Box::new(ConstArrayValidator {
            value: value.to_vec(),
            schema_path,
        }))
    }
}
impl Validate for ConstArrayValidator {
    #[inline]
    fn validate<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::constant_array(
                self.schema_path.clone(),
                instance_path.into(),
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
    schema_path: JSONPointer,
}
impl ConstBooleanValidator {
    #[inline]
    pub(crate) fn compile<'a>(value: bool, schema_path: JSONPointer) -> CompilationResult<'a> {
        Ok(Box::new(ConstBooleanValidator { value, schema_path }))
    }
}
impl Validate for ConstBooleanValidator {
    #[inline]
    fn validate<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::constant_boolean(
                self.schema_path.clone(),
                instance_path.into(),
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

struct ConstNullValidator {
    schema_path: JSONPointer,
}
impl ConstNullValidator {
    #[inline]
    pub(crate) fn compile<'a>(schema_path: JSONPointer) -> CompilationResult<'a> {
        Ok(Box::new(ConstNullValidator { schema_path }))
    }
}
impl Validate for ConstNullValidator {
    #[inline]
    fn validate<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::constant_null(
                self.schema_path.clone(),
                instance_path.into(),
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
    schema_path: JSONPointer,
}

impl ConstNumberValidator {
    #[inline]
    pub(crate) fn compile(original_value: &Number, schema_path: JSONPointer) -> CompilationResult {
        Ok(Box::new(ConstNumberValidator {
            original_value: original_value.clone(),
            value: original_value
                .as_f64()
                .expect("A JSON number will always be representable as f64"),
            schema_path,
        }))
    }
}

impl Validate for ConstNumberValidator {
    fn validate<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::constant_number(
                self.schema_path.clone(),
                instance_path.into(),
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
    schema_path: JSONPointer,
}

impl ConstObjectValidator {
    #[inline]
    pub(crate) fn compile(
        value: &Map<String, Value>,
        schema_path: JSONPointer,
    ) -> CompilationResult {
        Ok(Box::new(ConstObjectValidator {
            value: value.clone(),
            schema_path,
        }))
    }
}

impl Validate for ConstObjectValidator {
    fn validate<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::constant_object(
                self.schema_path.clone(),
                instance_path.into(),
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
    schema_path: JSONPointer,
}

impl ConstStringValidator {
    #[inline]
    pub(crate) fn compile(value: &str, schema_path: JSONPointer) -> CompilationResult {
        Ok(Box::new(ConstStringValidator {
            value: value.to_string(),
            schema_path,
        }))
    }
}

impl Validate for ConstStringValidator {
    fn validate<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::constant_string(
                self.schema_path.clone(),
                instance_path.into(),
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
pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    let schema_path = context.as_pointer_with("const");
    match schema {
        Value::Array(items) => Some(ConstArrayValidator::compile(items, schema_path)),
        Value::Bool(item) => Some(ConstBooleanValidator::compile(*item, schema_path)),
        Value::Null => Some(ConstNullValidator::compile(schema_path)),
        Value::Number(item) => Some(ConstNumberValidator::compile(item, schema_path)),
        Value::Object(map) => Some(ConstObjectValidator::compile(map, schema_path)),
        Value::String(string) => Some(ConstStringValidator::compile(string, schema_path)),
    }
}

#[cfg(test)]
mod tests {
    use crate::tests_util;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(&json!({"const": 1}), &json!(2), "/const")]
    #[test_case(&json!({"const": null}), &json!(3), "/const")]
    #[test_case(&json!({"const": false}), &json!(4), "/const")]
    #[test_case(&json!({"const": []}), &json!(5), "/const")]
    #[test_case(&json!({"const": {}}), &json!(6), "/const")]
    #[test_case(&json!({"const": ""}), &json!(7), "/const")]
    fn schema_path(schema: &Value, instance: &Value, expected: &str) {
        tests_util::assert_schema_path(schema, instance, expected)
    }
}
