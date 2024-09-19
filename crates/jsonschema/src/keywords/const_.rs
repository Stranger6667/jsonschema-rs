use crate::{
    compiler,
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::{helpers, CompilationResult},
    validator::Validate,
};
use serde_json::{Map, Number, Value};

use crate::paths::{JsonPointer, JsonPointerNode};

struct ConstArrayValidator {
    value: Vec<Value>,
    schema_path: JsonPointer,
}
impl ConstArrayValidator {
    #[inline]
    pub(crate) fn compile(value: &[Value], schema_path: JsonPointer) -> CompilationResult {
        Ok(Box::new(ConstArrayValidator {
            value: value.to_vec(),
            schema_path,
        }))
    }
}
impl Validate for ConstArrayValidator {
    #[inline]
    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &JsonPointerNode,
    ) -> ErrorIterator<'instance> {
        if self.is_valid(instance) {
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
    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::Array(instance_value) = instance {
            helpers::equal_arrays(&self.value, instance_value)
        } else {
            false
        }
    }
}

struct ConstBooleanValidator {
    value: bool,
    schema_path: JsonPointer,
}
impl ConstBooleanValidator {
    #[inline]
    pub(crate) fn compile<'a>(value: bool, schema_path: JsonPointer) -> CompilationResult<'a> {
        Ok(Box::new(ConstBooleanValidator { value, schema_path }))
    }
}
impl Validate for ConstBooleanValidator {
    #[inline]
    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &JsonPointerNode,
    ) -> ErrorIterator<'instance> {
        if self.is_valid(instance) {
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
    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::Bool(instance_value) = instance {
            &self.value == instance_value
        } else {
            false
        }
    }
}

struct ConstNullValidator {
    schema_path: JsonPointer,
}
impl ConstNullValidator {
    #[inline]
    pub(crate) fn compile<'a>(schema_path: JsonPointer) -> CompilationResult<'a> {
        Ok(Box::new(ConstNullValidator { schema_path }))
    }
}
impl Validate for ConstNullValidator {
    #[inline]
    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &JsonPointerNode,
    ) -> ErrorIterator<'instance> {
        if self.is_valid(instance) {
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
    fn is_valid(&self, instance: &Value) -> bool {
        instance.is_null()
    }
}

struct ConstNumberValidator {
    // This is saved in order to ensure that the error message is not altered by precision loss
    original_value: Number,
    value: f64,
    schema_path: JsonPointer,
}

impl ConstNumberValidator {
    #[inline]
    pub(crate) fn compile(original_value: &Number, schema_path: JsonPointer) -> CompilationResult {
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
    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &JsonPointerNode,
    ) -> ErrorIterator<'instance> {
        if self.is_valid(instance) {
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

    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::Number(item) = instance {
            (self.value - item.as_f64().expect("Always representable as f64")).abs() < f64::EPSILON
        } else {
            false
        }
    }
}

pub(crate) struct ConstObjectValidator {
    value: Map<String, Value>,
    schema_path: JsonPointer,
}

impl ConstObjectValidator {
    #[inline]
    pub(crate) fn compile(
        value: &Map<String, Value>,
        schema_path: JsonPointer,
    ) -> CompilationResult {
        Ok(Box::new(ConstObjectValidator {
            value: value.clone(),
            schema_path,
        }))
    }
}

impl Validate for ConstObjectValidator {
    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &JsonPointerNode,
    ) -> ErrorIterator<'instance> {
        if self.is_valid(instance) {
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

    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            helpers::equal_objects(&self.value, item)
        } else {
            false
        }
    }
}

pub(crate) struct ConstStringValidator {
    value: String,
    schema_path: JsonPointer,
}

impl ConstStringValidator {
    #[inline]
    pub(crate) fn compile(value: &str, schema_path: JsonPointer) -> CompilationResult {
        Ok(Box::new(ConstStringValidator {
            value: value.to_string(),
            schema_path,
        }))
    }
}

impl Validate for ConstStringValidator {
    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &JsonPointerNode,
    ) -> ErrorIterator<'instance> {
        if self.is_valid(instance) {
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

    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            &self.value == item
        } else {
            false
        }
    }
}

#[inline]
pub(crate) fn compile<'a>(
    ctx: &compiler::Context,
    _: &'a Map<String, Value>,
    schema: &'a Value,
) -> Option<CompilationResult<'a>> {
    let schema_path = ctx.as_pointer_with("const");
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
