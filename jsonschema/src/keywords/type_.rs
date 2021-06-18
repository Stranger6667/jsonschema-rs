use crate::{
    compilation::{context::CompilationContext, JSONSchema},
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    primitive_type::{PrimitiveType, PrimitiveTypesBitMap},
    validator::Validate,
};
use serde_json::{Map, Number, Value};
use std::convert::TryFrom;

use crate::paths::{InstancePath, JSONPointer};

pub(crate) struct MultipleTypesValidator {
    types: PrimitiveTypesBitMap,
    schema_path: JSONPointer,
}

impl MultipleTypesValidator {
    #[inline]
    pub(crate) fn compile(items: &[Value], schema_path: JSONPointer) -> CompilationResult {
        let mut types = PrimitiveTypesBitMap::new();
        for item in items {
            match item {
                Value::String(string) => {
                    if let Ok(primitive_type) = PrimitiveType::try_from(string.as_str()) {
                        types |= primitive_type;
                    } else {
                        return Err(ValidationError::schema(item));
                    }
                }
                _ => return Err(ValidationError::schema(item)),
            }
        }
        Ok(Box::new(MultipleTypesValidator { types, schema_path }))
    }
}

impl Validate for MultipleTypesValidator {
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        match instance {
            Value::Array(_) => self.types.contains_type(PrimitiveType::Array),
            Value::Bool(_) => self.types.contains_type(PrimitiveType::Boolean),
            Value::Null => self.types.contains_type(PrimitiveType::Null),
            Value::Number(num) => {
                self.types.contains_type(PrimitiveType::Number)
                    || (self.types.contains_type(PrimitiveType::Integer) && is_integer(num))
            }
            Value::Object(_) => self.types.contains_type(PrimitiveType::Object),
            Value::String(_) => self.types.contains_type(PrimitiveType::String),
        }
    }
    fn validate<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::multiple_type_error(
                self.schema_path.clone(),
                instance_path.into(),
                instance,
                self.types,
            ))
        }
    }
}

impl ToString for MultipleTypesValidator {
    fn to_string(&self) -> String {
        format!(
            "type: [{}]",
            self.types
                .into_iter()
                .map(|type_| format!("{}", type_))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

pub(crate) struct NullTypeValidator {
    schema_path: JSONPointer,
}

impl NullTypeValidator {
    #[inline]
    pub(crate) fn compile<'a>(schema_path: JSONPointer) -> CompilationResult<'a> {
        Ok(Box::new(NullTypeValidator { schema_path }))
    }
}

impl Validate for NullTypeValidator {
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        instance.is_null()
    }
    fn validate<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::single_type_error(
                self.schema_path.clone(),
                instance_path.into(),
                instance,
                PrimitiveType::Null,
            ))
        }
    }
}

impl ToString for NullTypeValidator {
    fn to_string(&self) -> String {
        "type: null".to_string()
    }
}

pub(crate) struct BooleanTypeValidator {
    schema_path: JSONPointer,
}

impl BooleanTypeValidator {
    #[inline]
    pub(crate) fn compile<'a>(schema_path: JSONPointer) -> CompilationResult<'a> {
        Ok(Box::new(BooleanTypeValidator { schema_path }))
    }
}

impl Validate for BooleanTypeValidator {
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        instance.is_boolean()
    }
    fn validate<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::single_type_error(
                self.schema_path.clone(),
                instance_path.into(),
                instance,
                PrimitiveType::Boolean,
            ))
        }
    }
}

impl ToString for BooleanTypeValidator {
    fn to_string(&self) -> String {
        "type: boolean".to_string()
    }
}

pub(crate) struct StringTypeValidator {
    schema_path: JSONPointer,
}

impl StringTypeValidator {
    #[inline]
    pub(crate) fn compile<'a>(schema_path: JSONPointer) -> CompilationResult<'a> {
        Ok(Box::new(StringTypeValidator { schema_path }))
    }
}

impl Validate for StringTypeValidator {
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        instance.is_string()
    }

    fn validate<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::single_type_error(
                self.schema_path.clone(),
                instance_path.into(),
                instance,
                PrimitiveType::String,
            ))
        }
    }
}
impl ToString for StringTypeValidator {
    fn to_string(&self) -> String {
        "type: string".to_string()
    }
}

pub(crate) struct ArrayTypeValidator {
    schema_path: JSONPointer,
}

impl ArrayTypeValidator {
    #[inline]
    pub(crate) fn compile<'a>(schema_path: JSONPointer) -> CompilationResult<'a> {
        Ok(Box::new(ArrayTypeValidator { schema_path }))
    }
}

impl Validate for ArrayTypeValidator {
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        instance.is_array()
    }

    fn validate<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::single_type_error(
                self.schema_path.clone(),
                instance_path.into(),
                instance,
                PrimitiveType::Array,
            ))
        }
    }
}

impl ToString for ArrayTypeValidator {
    fn to_string(&self) -> String {
        "type: array".to_string()
    }
}

pub(crate) struct ObjectTypeValidator {
    schema_path: JSONPointer,
}

impl ObjectTypeValidator {
    #[inline]
    pub(crate) fn compile<'a>(schema_path: JSONPointer) -> CompilationResult<'a> {
        Ok(Box::new(ObjectTypeValidator { schema_path }))
    }
}

impl Validate for ObjectTypeValidator {
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        instance.is_object()
    }
    fn validate<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::single_type_error(
                self.schema_path.clone(),
                instance_path.into(),
                instance,
                PrimitiveType::Object,
            ))
        }
    }
}

impl ToString for ObjectTypeValidator {
    fn to_string(&self) -> String {
        "type: object".to_string()
    }
}

pub(crate) struct NumberTypeValidator {
    schema_path: JSONPointer,
}

impl NumberTypeValidator {
    #[inline]
    pub(crate) fn compile<'a>(schema_path: JSONPointer) -> CompilationResult<'a> {
        Ok(Box::new(NumberTypeValidator { schema_path }))
    }
}

impl Validate for NumberTypeValidator {
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        instance.is_number()
    }
    fn validate<'a>(
        &self,
        config: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if self.is_valid(config, instance) {
            no_error()
        } else {
            error(ValidationError::single_type_error(
                self.schema_path.clone(),
                instance_path.into(),
                instance,
                PrimitiveType::Number,
            ))
        }
    }
}
impl ToString for NumberTypeValidator {
    fn to_string(&self) -> String {
        "type: number".to_string()
    }
}
pub(crate) struct IntegerTypeValidator {
    schema_path: JSONPointer,
}

impl IntegerTypeValidator {
    #[inline]
    pub(crate) fn compile<'a>(schema_path: JSONPointer) -> CompilationResult<'a> {
        Ok(Box::new(IntegerTypeValidator { schema_path }))
    }
}

impl Validate for IntegerTypeValidator {
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::Number(num) = instance {
            is_integer(num)
        } else {
            false
        }
    }
    fn validate<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::single_type_error(
                self.schema_path.clone(),
                instance_path.into(),
                instance,
                PrimitiveType::Integer,
            ))
        }
    }
}

impl ToString for IntegerTypeValidator {
    fn to_string(&self) -> String {
        "type: integer".to_string()
    }
}

fn is_integer(num: &Number) -> bool {
    num.is_u64() || num.is_i64() || num.as_f64().expect("Always valid").fract() == 0.
}

#[inline]
pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    let schema_path = context.as_pointer_with("type");
    match schema {
        Value::String(item) => compile_single_type(item.as_str(), schema_path),
        Value::Array(items) => {
            if items.len() == 1 {
                if let Some(Value::String(item)) = items.iter().next() {
                    compile_single_type(item.as_str(), schema_path)
                } else {
                    Some(Err(ValidationError::schema(schema)))
                }
            } else {
                Some(MultipleTypesValidator::compile(items, schema_path))
            }
        }
        _ => Some(Err(ValidationError::schema(schema))),
    }
}

fn compile_single_type(item: &str, schema_path: JSONPointer) -> Option<CompilationResult> {
    match PrimitiveType::try_from(item) {
        Ok(PrimitiveType::Array) => Some(ArrayTypeValidator::compile(schema_path)),
        Ok(PrimitiveType::Boolean) => Some(BooleanTypeValidator::compile(schema_path)),
        Ok(PrimitiveType::Integer) => Some(IntegerTypeValidator::compile(schema_path)),
        Ok(PrimitiveType::Null) => Some(NullTypeValidator::compile(schema_path)),
        Ok(PrimitiveType::Number) => Some(NumberTypeValidator::compile(schema_path)),
        Ok(PrimitiveType::Object) => Some(ObjectTypeValidator::compile(schema_path)),
        Ok(PrimitiveType::String) => Some(StringTypeValidator::compile(schema_path)),
        Err(()) => Some(Err(ValidationError::null_schema())),
    }
}

#[cfg(test)]
mod tests {
    use crate::tests_util;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(&json!({"type": "array"}), &json!(1), "/type")]
    #[test_case(&json!({"type": "boolean"}), &json!(1), "/type")]
    #[test_case(&json!({"type": "integer"}), &json!("f"), "/type")]
    #[test_case(&json!({"type": "null"}), &json!(1), "/type")]
    #[test_case(&json!({"type": "number"}), &json!("f"), "/type")]
    #[test_case(&json!({"type": "object"}), &json!(1), "/type")]
    #[test_case(&json!({"type": "string"}), &json!(1), "/type")]
    #[test_case(&json!({"type": ["string", "object"]}), &json!(1), "/type")]
    fn schema_path(schema: &Value, instance: &Value, expected: &str) {
        tests_util::assert_schema_path(schema, instance, expected)
    }
}
